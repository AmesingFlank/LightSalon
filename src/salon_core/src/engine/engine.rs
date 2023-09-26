use std::{collections::HashMap, sync::Arc};

use crate::{
    engine::ExposureOp,
    image::Image,
    ir::{Id, Input, Module, Op, Value},
    runtime::Runtime,
};

pub struct Engine {
    pub runtime: Arc<Runtime>,
    pub exposure_op: ExposureOp,
    pub value_store: HashMap<Id, Value>,
}

impl Engine {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let exposure_op = ExposureOp::new(runtime.clone());
        Engine {
            runtime,
            exposure_op,
            value_store: HashMap::new(),
        }
    }

    pub fn execute_module(&mut self, module: &Module, input_img: Arc<Image>) -> Arc<Image> {
        self.value_store.clear();
        let ops = module.ops();
        for op in ops {
            match op {
                Op::Input(ref input) => {
                    self.value_store
                        .insert(input.result, Value::Image(input_img.clone()));
                }
                Op::ExposureAdjust(ref exposure) => {

                }
            }
        }

        let output_id = module.output_id().expect("expecting an output id");
        let output_value = self
            .value_store
            .get(&output_id)
            .expect("cannot find output");
        match output_value {
            Value::Image(ref img) => img.clone(),
            _ => {
                panic!("expecting an image")
            }
        }
    }
}
