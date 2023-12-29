use std::sync::Arc;

use crate::{
    engine::{Engine, ExecutionContext},
    image::Image,
    runtime::{MipmapGenerator, Runtime},
};

use super::{
    ir_generator::{to_ir_module, IdStore},
    DataForEditor, Edit, EditResult,
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

        let output_value = value_map
            .get(&id_store.output_id)
            .expect("cannot find output");
        let output_image = output_value.as_image().clone();
        self.mipmap_generator.generate(&output_image);


        let editor_data_buffer = value_map
            .get(&id_store.data_for_editor_id)
            .expect("cannot find data for editor")
            .as_buffer();

        let mut result = EditResult::new_empty();

        let data_for_editor = DataForEditor::from_buffer(&editor_data_buffer, &self.runtime);
        // println!("");
        // let mut sum = 0u32;
        // for i in 0..stats.histogram_final.num_bins as usize {
        //     print!("{x} ", x=stats.histogram_final.r[i]);
        //     sum = sum + stats.histogram_final.r[i];
        // }
        // println!("");
        // println!("num_bins={num_bins}",num_bins=stats.histogram_final.num_bins);
        // println!("sum={sum}",sum=sum);
        // println!("");
        result.data_for_editor = Some(data_for_editor);

        result.final_image = Some(output_image);

        self.current_result = Some(result);
    }
}
