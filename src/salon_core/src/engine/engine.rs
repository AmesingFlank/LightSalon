use std::{collections::HashMap, sync::Arc};

use crate::{
    image::Image,
    ir::{Id, Input, Module, Op, Value},
    runtime::Runtime,
};

use super::{
    op_impl_collection::OpImplCollection,
    ops::{
        brightness::{self, BrightnessAdjustImpl},
        exposure::ExposureAdjustImpl,
    },
    value_store::ValueStore,
};

pub struct Engine {
    pub runtime: Arc<Runtime>,
    pub op_impls: OpImplCollection,
    pub value_store: ValueStore,
}

impl Engine {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Engine {
            runtime,
            op_impls: OpImplCollection::new(),
            value_store: ValueStore::new(),
        }
    }

    pub fn execute_module(&mut self, module: &Module, input_img: Arc<Image>) -> Arc<Image> {
        self.ensure_op_impls(module);
        let ops = module.ops();
        for op in ops {
            match op {
                Op::Input(ref input) => {
                    self.value_store
                        .map
                        .insert(input.result, Value::Image(input_img.clone()));
                }
                Op::ExposureAdjust(ref exposure) => {
                    self.op_impls
                        .exposure
                        .as_mut()
                        .unwrap()
                        .apply(exposure, &mut self.value_store);
                }
                Op::BrightnessAdjust(ref brightness) => {
                    self.op_impls
                        .brightness
                        .as_mut()
                        .unwrap()
                        .apply(brightness, &mut self.value_store);
                }
            }
        }

        let output_id = module.output_id().expect("expecting an output id");
        let output_value = self
            .value_store
            .map
            .get(&output_id)
            .expect("cannot find output");
        let output_image =  output_value.as_image().clone();
        self.runtime.ensure_mipmap(&output_image.as_ref());
        output_image
    }

    fn ensure_op_impls(&mut self, module: &Module) {
        let ops = module.ops();
        for op in ops {
            match op {
                Op::Input(_) => {}
                Op::ExposureAdjust(_) => {
                    self.op_impls.exposure = Some(ExposureAdjustImpl::new(self.runtime.clone()))
                }
                Op::BrightnessAdjust(_) => {
                    self.op_impls.brightness = Some(BrightnessAdjustImpl::new(self.runtime.clone()))
                }
            }
        }
    }
}
