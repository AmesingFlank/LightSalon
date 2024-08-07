use std::{collections::HashMap, sync::Arc};

use crate::{
    engine::{common::ImageHistogram, Engine, ExecutionContext},
    library::LibraryImageIdentifier,
    runtime::{BufferReader, Image, Runtime, Toolbox},
    services::{edit_writer::EditWriterService, services::Services},
};

use super::{
    ir_generator::{to_ir_module, IdStore, IrGenerationOptions},
    Edit, EditResult, MaskedEditResult,
};

pub type EditHistory = Vec<Edit>;

pub struct Editor {
    engine: Engine,
    engine_execution_context: ExecutionContext,

    current_image_identifier: Option<LibraryImageIdentifier>,
    edit_contexts: HashMap<LibraryImageIdentifier, EditContext>,

    runtime: Arc<Runtime>,
    toolbox: Arc<Toolbox>,
    services: Arc<Services>,
}

pub struct EditContext {
    input_image: Option<Arc<Image>>,
    edit_history: EditHistory,
    current_edit_index: usize,

    // an edit that is being actively modified
    // (e.g. as the user drags the slider, the temporary edit state)
    transient_edit: Option<Edit>,
    pub current_result: Option<EditResult>,

    // full size result that includes framing
    // (not equal to current_result.final_image, which might not be full-size)
    pub current_full_size_editted_image: Option<Arc<Image>>,
}

impl EditContext {
    pub fn input_image(&self) -> &Arc<Image> {
        &self.input_image.as_ref().unwrap()
    }

    pub fn clone_edit_history(&self) -> EditHistory {
        let mut history = Vec::new();
        for i in 0..=self.current_edit_index {
            history.push(self.edit_history[i].clone());
        }
        history
    }

    pub fn current_edit_ref(&self) -> &Edit {
        &self.edit_history[self.current_edit_index]
    }

    pub fn transient_edit_ref(&self) -> &Edit {
        if self.transient_edit.is_none() {
            self.current_edit_ref()
        } else {
            self.transient_edit.as_ref().unwrap()
        }
    }

    // returns true iff an update was made
    fn update_transient_edit(&mut self, transient_edit: Edit) -> bool {
        let needs_update: bool;
        if let Some(ref curr_transient_edit) = self.transient_edit {
            needs_update = *curr_transient_edit != transient_edit;
        } else {
            needs_update = self.edit_history[self.current_edit_index] != transient_edit;
        }
        if needs_update {
            self.transient_edit = Some(transient_edit);
        }
        needs_update
    }

    // returns true iff edit history is updated
    fn commit_transient_edit(&mut self) -> bool {
        let mut needs_commit = false;
        if let Some(ref transient) = self.transient_edit {
            if *transient != self.edit_history[self.current_edit_index] {
                needs_commit = true;
            }
        }
        if needs_commit {
            while self.current_edit_index < self.edit_history.len() - 1 {
                self.edit_history.pop();
            }
            self.edit_history.push(self.transient_edit.take().unwrap());
            self.current_edit_index = self.edit_history.len() - 1;

            self.current_full_size_editted_image = None;
        }
        self.transient_edit = None;
        needs_commit
    }

    pub fn can_undo(&self) -> bool {
        self.current_edit_index > 0
    }

    pub fn can_redo(&self) -> bool {
        self.current_edit_index < self.edit_history.len() - 1
    }

    // returns whether or not an undo happened
    fn maybe_undo(&mut self) -> bool {
        if self.current_edit_index > 0 {
            self.current_edit_index -= 1;
            self.transient_edit = None;
            self.current_full_size_editted_image = None;
            true
        } else {
            false
        }
    }

    // returns whether or not a redo happened
    fn maybe_redo(&mut self) -> bool {
        if self.current_edit_index < self.edit_history.len() - 1 {
            self.current_edit_index += 1;
            self.transient_edit = None;
            self.current_full_size_editted_image = None;
            true
        } else {
            false
        }
    }

    pub fn override_resize_factor(&mut self, new_resize_factor: f32) {
        if let Some(ref mut e) = self.transient_edit {
            e.resize_factor = Some(new_resize_factor);
        }
        for e in self.edit_history.iter_mut() {
            e.resize_factor = Some(new_resize_factor);
        }
    }
}

impl Editor {
    pub fn new(runtime: Arc<Runtime>, toolbox: Arc<Toolbox>, services: Arc<Services>) -> Self {
        let engine = Engine::new(runtime.clone(), toolbox.clone());
        Editor {
            engine,
            current_image_identifier: None,
            edit_contexts: HashMap::new(),

            engine_execution_context: ExecutionContext::new(),
            runtime,
            toolbox,
            services,
        }
    }

    pub fn set_current_image(&mut self, identifier: LibraryImageIdentifier, image: Arc<Image>) {
        if let Some(ref curr_identifier) = self.current_image_identifier {
            if *curr_identifier == identifier {
                return;
            }
        }

        if let Some(context) = self.edit_contexts.get_mut(&identifier) {
            context.input_image = Some(image)
        } else {
            let mut edit = Edit::trivial();
            if let Some(image_path) = identifier.get_path() {
                if let Some(edit_path) =
                    EditWriterService::get_edit_path_for_image_path(&image_path)
                {
                    if edit_path.exists() {
                        if let Ok(edit_json_str) = std::fs::read_to_string(&edit_path) {
                            if let Ok(saved_edit) =
                                serde_json::from_str::<Edit>(edit_json_str.as_str())
                            {
                                edit = saved_edit;
                            }
                        }
                    }
                }
            }
            let new_context = EditContext {
                input_image: Some(image),
                edit_history: vec![edit],
                current_edit_index: 0,
                transient_edit: None,
                current_result: None,
                current_full_size_editted_image: None,
            };
            self.edit_contexts.insert(identifier.clone(), new_context);
        }

        self.current_image_identifier = Some(identifier);
        self.execute_current_edit();
    }

    pub fn current_image_identifier(&self) -> Option<LibraryImageIdentifier> {
        self.current_image_identifier.clone()
    }

    pub fn clear_current_image(&mut self) {
        self.current_image_identifier = None;
    }

    pub fn current_edit_context_ref(&self) -> Option<&EditContext> {
        let identifier = self.current_image_identifier.as_ref()?;
        self.edit_contexts.get(identifier)
    }

    pub fn current_edit_context_mut(&mut self) -> Option<&mut EditContext> {
        let identifier = self.current_image_identifier.as_ref()?;
        self.edit_contexts.get_mut(identifier)
    }

    pub fn update_transient_edit(&mut self, transient_edit: Edit, execute: bool) {
        let updated = self
            .current_edit_context_mut()
            .unwrap()
            .update_transient_edit(transient_edit);
        if updated && execute {
            self.execute_transient_edit();
        }
    }

    pub fn commit_transient_edit(&mut self, execute: bool) -> bool {
        let committed = self
            .current_edit_context_mut()
            .unwrap()
            .commit_transient_edit();
        if committed {
            self.update_current_edit_in_filesystem();
        }
        if committed && execute {
            self.execute_current_edit();
        }
        committed
    }

    fn update_current_edit_in_filesystem(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(ref identifier) = self.current_image_identifier {
            if let Some(path) = identifier.get_path() {
                if let Some(edit_context) = self.current_edit_context_ref() {
                    self.services
                        .edit_writer
                        .request_update(edit_context.current_edit_ref().clone(), path);
                }
            }
        }
    }

    pub fn can_undo(&mut self) -> bool {
        if let Some(context) = self.current_edit_context_ref() {
            context.can_undo()
        } else {
            false
        }
    }

    pub fn can_redo(&mut self) -> bool {
        if let Some(context) = self.current_edit_context_ref() {
            context.can_redo()
        } else {
            false
        }
    }

    pub fn maybe_undo(&mut self) -> bool {
        if let Some(context) = self.current_edit_context_mut() {
            if context.maybe_undo() {
                self.execute_current_edit();
                self.update_current_edit_in_filesystem();
                return true;
            }
        }
        false
    }

    pub fn maybe_redo(&mut self) -> bool {
        if let Some(context) = self.current_edit_context_mut() {
            if context.maybe_redo() {
                self.execute_current_edit();
                self.update_current_edit_in_filesystem();
                return true;
            }
        }
        false
    }

    pub fn execute_current_edit(&mut self) {
        let (module, id_store) = to_ir_module(
            self.current_edit_context_ref().unwrap().current_edit_ref(),
            &IrGenerationOptions {
                compute_histogram: true,
            },
        );
        let image = self
            .current_edit_context_ref()
            .unwrap()
            .input_image()
            .clone();
        self.engine
            .execute_module(&module, image, &mut self.engine_execution_context);
        self.current_edit_context_mut().unwrap().current_result =
            Some(self.collect_result(&id_store));
    }

    fn execute_transient_edit(&mut self) {
        let (module, id_store) = to_ir_module(
            self.current_edit_context_ref()
                .unwrap()
                .transient_edit_ref(),
            &IrGenerationOptions {
                compute_histogram: true,
            },
        );
        let image = self
            .current_edit_context_ref()
            .unwrap()
            .input_image()
            .clone();
        self.engine
            .execute_module(&module, image, &mut self.engine_execution_context);
        self.current_edit_context_mut().unwrap().current_result =
            Some(self.collect_result(&id_store));
    }

    pub fn get_full_size_edit(&self) -> Edit {
        let edit = self
            .current_edit_context_ref()
            .unwrap()
            .current_edit_ref()
            .clone();
        Edit {
            resize_factor: None,
            ..edit
        }
    }

    pub fn get_full_size_editted_image(&mut self) -> Arc<Image> {
        if let Some(ref result) = self
            .current_edit_context_ref()
            .unwrap()
            .current_full_size_editted_image
        {
            return result.clone();
        }

        let edit = self.get_full_size_edit();

        let (module, id_store) = to_ir_module(
            &edit,
            &IrGenerationOptions {
                compute_histogram: false,
            },
        );

        self.engine.execute_module(
            &module,
            self.current_edit_context_ref()
                .unwrap()
                .input_image()
                .clone(),
            &mut self.engine_execution_context,
        );
        let result = self.collect_result(&id_store);
        let full_size_result_image = result.final_image;

        self.current_edit_context_mut()
            .unwrap()
            .current_full_size_editted_image = Some(full_size_result_image.clone());
        full_size_result_image
    }

    fn collect_result(&mut self, id_store: &IdStore) -> EditResult {
        let mut histogram_initial_value = None;
        if let Some(context) = self.current_edit_context_mut() {
            if let Some(ref mut current_result) = context.current_result {
                if let Some(ref mut current_histogram) = current_result.histogram_final {
                    current_histogram.poll_value();
                    histogram_initial_value = current_histogram.take_value();
                }
            }
        }

        let value_map = &self.engine_execution_context.value_store.map;

        let final_image_value = value_map
            .get(&id_store.final_image)
            .expect("cannot find output");
        let final_image = final_image_value.as_image().clone();
        self.toolbox.generate_mipmap(&final_image);

        let mut final_histogram = None;
        if let Some(ref final_histogram_id) = id_store.final_histogram {
            let final_histogram_buffer = value_map
                .get(final_histogram_id)
                .expect("cannot find data for editor")
                .as_buffer()
                .clone();

            final_histogram = Some(BufferReader::new(
                self.runtime.clone(),
                final_histogram_buffer,
                histogram_initial_value,
                Box::new(|v| ImageHistogram::from_u32_slice(v.as_slice())),
            ));
        }

        let mut masked_edit_results = Vec::new();

        for masked_edit_id_store in id_store.masked_edit_id_stores.iter() {
            let mask = value_map
                .get(&masked_edit_id_store.mask_id)
                .expect("cannot find mask")
                .as_image()
                .clone();
            let result_image = value_map
                .get(&masked_edit_id_store.result_image_id)
                .expect("cannot find result image")
                .as_image()
                .clone();
            let mut mask_terms = Vec::new();
            for term_id in masked_edit_id_store.term_ids.iter() {
                let term = value_map
                    .get(&term_id)
                    .expect("cannot find term")
                    .as_image()
                    .clone();
                mask_terms.push(term)
            }
            masked_edit_results.push(MaskedEditResult {
                mask,
                mask_terms,
                result_image,
            })
        }

        let geometry_only = value_map
            .get(&id_store.geometry_only)
            .expect("cannot find geometry-applied image")
            .as_image()
            .clone();
        self.toolbox.generate_mipmap(&geometry_only);

        let before_framing = value_map
            .get(&id_store.before_framing)
            .expect("cannot find image before framing")
            .as_image()
            .clone();
        self.toolbox.generate_mipmap(&before_framing);

        EditResult {
            final_image,
            geometry_only,
            before_framing,
            histogram_final: final_histogram,
            masked_edit_results,
        }
    }
}
