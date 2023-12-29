use std::sync::Arc;

use crate::{
    engine::{common::ImageHistogram, Engine, ExecutionContext},
    image::Image,
    runtime::{MipmapGenerator, Runtime},
};

use super::{
    ir_generator::{to_ir_module, IdStore},
    Edit, EditResult,
};

pub struct Editor {
    pub current_edit: Edit,
    pub current_input_image: Option<Arc<Image>>,
    pub current_result: Option<EditResult>,
    engine_execution_context: ExecutionContext,
    runtime: Arc<Runtime>,
    mipmap_generator: MipmapGenerator,
}

impl Editor {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let mipmap_generator = MipmapGenerator::new(runtime.clone());
        Editor {
            current_edit: Edit::new(),
            current_input_image: None,
            current_result: None,
            engine_execution_context: ExecutionContext::new(),
            runtime,
            mipmap_generator,
        }
    }

    pub fn reset_state(&mut self) {
        self.current_edit = Edit::new();
    }

    pub fn execute_edit(&mut self, engine: &mut Engine) {
        if let Some(ref img) = self.current_input_image {
            let (module, id_store) = to_ir_module(&self.current_edit);
            engine.execute_module(&module, img.clone(), &mut self.engine_execution_context);
            self.collect_result(&id_store);
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

        let mut masks = Vec::new();

        for mask_id in id_store.masks.iter() {
            let mask_image = value_map.get(&mask_id).expect("cannot find mask").as_image().clone();
            masks.push(mask_image)
        }

        let result = EditResult {
            final_image: output_image,
            histogram_final: final_hist,
            masks
        };

        self.current_result = Some(result);
    }
}
