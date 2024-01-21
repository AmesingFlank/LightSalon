use std::sync::Arc;

use crate::{
    engine::{common::ImageHistogram, Engine, ExecutionContext},
    runtime::{Buffer, Image, MipmapGenerator, Runtime},
};

use super::{
    ir_generator::{to_ir_module, IdStore},
    Edit, EditResult, MaskedEditResult,
};

pub struct Editor {
    engine: Engine,
    engine_execution_context: ExecutionContext,

    pub current_input_image: Option<Arc<Image>>,

    edit_history: Vec<Edit>,
    current_edit_index: usize,

    // an edit that is being actively modified
    // (e.g. as the user drags the slider, the temporary edit state)
    transient_edit: Option<Edit>,

    pub current_result: Option<EditResult>,
    runtime: Arc<Runtime>,
    mipmap_generator: MipmapGenerator,
}

impl Editor {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let engine = Engine::new(runtime.clone());
        let mipmap_generator = MipmapGenerator::new(runtime.clone());
        Editor {
            engine,
            current_input_image: None,

            edit_history: vec![Edit::new()],
            current_edit_index: 0,
            transient_edit: None,

            current_result: None,
            engine_execution_context: ExecutionContext::new(),
            runtime,
            mipmap_generator,
        }
    }

    pub fn clear_edit_history(&mut self) {
        self.edit_history = vec![Edit::new()];
        self.current_edit_index = 0;
    }

    fn get_current_edit(&self) -> Edit {
        self.edit_history[self.current_edit_index].clone()
    }

    pub fn clone_transient_edit(&self) -> Edit {
        if self.transient_edit.is_none() {
            self.get_current_edit()
        } else {
            self.transient_edit.as_ref().unwrap().clone()
        }
    }

    pub fn update_transient_edit(&mut self, transient_edit: Edit, execute: bool) {
        let mut needs_update = false;
        if let Some(ref curr_transient_edit) = self.transient_edit {
            needs_update = (*curr_transient_edit != transient_edit);
        } else {
            needs_update = (self.edit_history[self.current_edit_index] != transient_edit);
        }
        if needs_update {
            self.transient_edit = Some(transient_edit);
            if execute {
                self.execute_transient_edit();
            }
        }
    }

    pub fn commit_transient_edit(&mut self) {
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
            self.execute_current_edit();
        }
        self.transient_edit = None;
    }

    pub fn can_undo(&self) -> bool {
        self.current_edit_index > 0
    }

    pub fn can_redo(&self) -> bool {
        self.current_edit_index < self.edit_history.len() - 1
    }

    pub fn maybe_undo(&mut self) {
        if self.current_edit_index > 0 {
            self.current_edit_index -= 1;
            self.execute_current_edit();
            self.transient_edit = None;
        }
    }

    pub fn maybe_redo(&mut self) {
        if self.current_edit_index < self.edit_history.len() - 1 {
            self.current_edit_index += 1;
            self.execute_current_edit();
            self.transient_edit = None;
        }
    }

    pub fn execute_current_edit(&mut self) {
        if let Some(ref img) = self.current_input_image {
            let (module, id_store) = to_ir_module(&self.edit_history[self.current_edit_index]);
            self.engine
                .execute_module(&module, img.clone(), &mut self.engine_execution_context);
            self.collect_result(&id_store);
        }
    }

    fn execute_transient_edit(&mut self) {
        if let Some(ref e) = self.transient_edit {
            if let Some(ref img) = self.current_input_image {
                let (module, id_store) = to_ir_module(e);
                self.engine.execute_module(
                    &module,
                    img.clone(),
                    &mut self.engine_execution_context,
                );
                self.collect_result(&id_store);
            }
        }
    }

    fn collect_result(&mut self, id_store: &IdStore) {
        let value_map = &self.engine_execution_context.value_store.map;

        let output_value = value_map.get(&id_store.output).expect("cannot find output");
        let output_image = output_value.as_image().clone();
        self.mipmap_generator.generate(&output_image);

        let final_histogram_buffer = value_map
            .get(&id_store.final_histogram)
            .expect("cannot find data for editor")
            .as_buffer();
        let final_histogram_buffer_data: Vec<u32> =
            self.runtime.read_buffer(&final_histogram_buffer);

        let final_hist = ImageHistogram::from_u32_slice(final_histogram_buffer_data.as_slice());

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

        let result = EditResult {
            final_image: output_image,
            histogram_final: final_hist,
            masked_edit_results,
        };

        self.current_result = Some(result);
    }
}
