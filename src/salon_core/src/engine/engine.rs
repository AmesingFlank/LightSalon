use std::{collections::HashMap, sync::Arc};

use crate::{
    image::Image,
    ir::{Id, Input, Module, Op, Value},
    runtime::Runtime,
};

use super::{op_impl_collection::OpImplCollection, ops::exposure::ExposureAdjustImpl};

pub struct Engine {
    pub runtime: Arc<Runtime>,
    pub op_impls: OpImplCollection,
    pub value_store: HashMap<Id, Value>,
}

impl Engine {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Engine {
            runtime,
            op_impls: OpImplCollection::new(),
            value_store: HashMap::new(),
        }
    }

    pub fn execute_module(&mut self, module: &Module, input_img: Arc<Image>) -> Arc<Image> {
        self.ensure_op_impls(module);
        let ops = module.ops();
        for op in ops {
            match op {
                Op::Input(ref input) => {
                    self.value_store
                        .insert(input.result, Value::Image(input_img.clone()));
                }
                Op::ExposureAdjust(ref exposure) => {
                    self.op_impls
                        .exposure
                        .as_mut()
                        .unwrap()
                        .apply(exposure, &mut self.value_store);
                }
            }
        }

        let output_id = module.output_id().expect("expecting an output id");
        let output_value = self
            .value_store
            .get(&output_id)
            .expect("cannot find output");
        output_value.as_image().clone()
    }

    fn ensure_op_impls(&mut self, module: &Module) {
        let ops = module.ops();
        for op in ops {
            match op {
                Op::Input(_) => {}
                Op::ExposureAdjust(_) => {
                    self.op_impls.exposure = Some(ExposureAdjustImpl::new(self.runtime.clone()))
                }
            }
        }
    }
}
