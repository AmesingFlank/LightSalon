use std::{
    collections::HashMap,
    sync::{mpsc::Receiver, Arc},
};

use crate::{
    engine::{common::ImageHistogram, Engine, ExecutionContext},
    library::LibraryImageIdentifier,
    runtime::{Buffer, BufferReader, Image, Runtime, Toolbox},
};

use super::{
    ir_generator::{to_ir_module, IdStore},
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
}

pub struct EditContext {
    input_image: Option<Arc<Image>>,
    edit_history: EditHistory,
    current_edit_index: usize,

    // an edit that is being actively modified
    // (e.g. as the user drags the slider, the temporary edit state)
    transient_edit: Option<Edit>,
    pub current_result: Option<EditResult>,
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
        let mut needs_update = false;
        if let Some(ref curr_transient_edit) = self.transient_edit {
            needs_update = (*curr_transient_edit != transient_edit);
        } else {
            needs_update = (self.edit_history[self.current_edit_index] != transient_edit);
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
    pub fn new(runtime: Arc<Runtime>, toolbox: Arc<Toolbox>) -> Self {
        let engine = Engine::new(runtime.clone(), toolbox.clone());
        Editor {
            engine,
            current_image_identifier: None,
            edit_contexts: HashMap::new(),

            engine_execution_context: ExecutionContext::new(),
            runtime,
            toolbox,
        }
    }

    pub fn set_current_image(&mut self, identifier: LibraryImageIdentifier, image: Arc<Image>) {
        if let Some(ref curr_identifier) = self.current_image_identifier {
            if *curr_identifier == identifier {
                return;
            }
        }

        if let Some(context) = self.edit_contexts.get_mut(&identifier) {
            // update the image, in case it was None before (e.g. if the context is populated from a persistent state)
            context.input_image = Some(image)
        } else {
            let new_context = EditContext {
                input_image: Some(image),
                edit_history: vec![Edit::trivial()],
                current_edit_index: 0,
                transient_edit: None,
                current_result: None,
            };
            self.edit_contexts.insert(identifier.clone(), new_context);
        }

        self.current_image_identifier = Some(identifier);
        self.execute_current_edit();
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

    pub fn commit_transient_edit(&mut self, execute: bool) {
        let committed = self
            .current_edit_context_mut()
            .unwrap()
            .commit_transient_edit();
        if committed && execute {
            self.execute_current_edit();
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
                return true;
            }
        }
        false
    }

    pub fn maybe_redo(&mut self) -> bool {
        if let Some(context) = self.current_edit_context_mut() {
            if context.maybe_redo() {
                self.execute_current_edit();
                return true;
            }
        }
        false
    }

    pub fn execute_current_edit(&mut self) {
        let (module, id_store) =
            to_ir_module(self.current_edit_context_ref().unwrap().current_edit_ref());
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

    pub fn execute_current_edit_original_size(&mut self) -> EditResult {
        let mut edit = self
            .current_edit_context_ref()
            .unwrap()
            .current_edit_ref()
            .clone();
        edit.resize_factor = None;
        let image = self
            .current_edit_context_ref()
            .unwrap()
            .input_image()
            .clone();
        let (module, id_store) = to_ir_module(&edit);
        self.engine
            .execute_module(&module, image, &mut self.engine_execution_context);
        self.collect_result(&id_store)
    }

    fn collect_result(&mut self, id_store: &IdStore) -> EditResult {
        let mut histogram_initial_value = None;
        if let Some(context) = self.current_edit_context_mut() {
            if let Some(ref mut current_result) = context.current_result {
                let current_histogram = &mut current_result.histogram_final;
                current_histogram.poll_value();
                histogram_initial_value = current_histogram.take_value();
            }
        }

        let value_map = &self.engine_execution_context.value_store.map;

        let final_image_value = value_map.get(&id_store.final_image).expect("cannot find output");
        let final_image = final_image_value.as_image().clone();
        self.toolbox.generate_mipmap(&final_image);

        let final_histogram_buffer = value_map
            .get(&id_store.final_histogram)
            .expect("cannot find data for editor")
            .as_buffer();

        let final_histogram = BufferReader::new(
            self.runtime.clone(),
            final_histogram_buffer.clone(),
            histogram_initial_value,
            Box::new(|v| ImageHistogram::from_u32_slice(v.as_slice())),
        );

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

        EditResult {
            final_image,
            geometry_only,
            histogram_final: final_histogram,
            masked_edit_results,
        }
    }

    pub fn get_persistent_state(&self) -> EditorPersistentState {
        let mut edit_context_states = Vec::new();
        for (identifier, context) in self.edit_contexts.iter() {
            let state = EditContextPersistentState {
                identifier: identifier.clone(),
                current_edit: context.current_edit_ref().clone(),
            };
            edit_context_states.push(state);
        }
        EditorPersistentState {
            edit_context_states,
        }
    }

    pub fn load_persistent_state(&mut self, state: EditorPersistentState) {
        for edit_context_state in state.edit_context_states {
            let new_context = EditContext {
                input_image: None,
                edit_history: vec![edit_context_state.current_edit],
                current_edit_index: 0,
                transient_edit: None,
                current_result: None,
            };
            self.edit_contexts
                .insert(edit_context_state.identifier, new_context);
        }
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct EditorPersistentState {
    edit_context_states: Vec<EditContextPersistentState>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct EditContextPersistentState {
    identifier: LibraryImageIdentifier,
    current_edit: Edit,
}
