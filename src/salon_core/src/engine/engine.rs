use std::{collections::HashMap, sync::Arc};

use crate::{
    image::Image,
    ir::{Id, InputOp, Module, Op, Value},
    runtime::Runtime,
};

use super::{
    op_impl_collection::OpImplCollection,
    ops::{
        exposure::AdjustExposureImpl,
        histogram::{self, ComputeHistogramImpl},
        saturation::{self, AdjustSaturationImpl},
    },
    result::ProcessResult,
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

    pub fn execute_module(&mut self, module: &Module, input_img: Arc<Image>) -> ProcessResult {
        let mut result = ProcessResult::new_empty();
        self.reset_op_impls(module);
        let ops = module.ops();
        for op in ops {
            match op {
                Op::Input(ref input) => {
                    self.value_store
                        .map
                        .insert(input.result, Value::Image(input_img.clone()));
                }
                Op::AdjustExposure(ref exposure) => {
                    self.op_impls
                        .exposure
                        .as_mut()
                        .unwrap()
                        .apply(exposure, &mut self.value_store);
                }
                Op::AdjustSaturation(ref saturation) => {
                    self.op_impls
                        .saturation
                        .as_mut()
                        .unwrap()
                        .apply(saturation, &mut self.value_store);
                }
                Op::ComputeHistogram(ref histogram) => {
                    self.op_impls
                        .histogram
                        .as_mut()
                        .unwrap()
                        .apply(histogram, &mut self.value_store);
                }
            }
        }

        let output_id = module.output_id().expect("expecting an output id");
        let output_value = self
            .value_store
            .map
            .get(&output_id)
            .expect("cannot find output");
        let output_image = output_value.as_image().clone();
        self.runtime.ensure_mipmap(&output_image.as_ref());

        result.final_image = Some(output_image);
        result
    }

    fn reset_op_impls(&mut self, module: &Module) {
        let ops = module.ops();
        for op in ops {
            match op {
                Op::Input(_) => {}
                Op::AdjustExposure(_) => {
                    if self.op_impls.exposure.is_none() {
                        self.op_impls.exposure = Some(AdjustExposureImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.exposure.as_mut().unwrap().reset();
                }
                Op::AdjustSaturation(_) => {
                    if self.op_impls.saturation.is_none() {
                        self.op_impls.saturation =
                            Some(AdjustSaturationImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.saturation.as_mut().unwrap().reset();
                }
                Op::ComputeHistogram(_) => {
                    if self.op_impls.histogram.is_none() {
                        self.op_impls.histogram =
                            Some(ComputeHistogramImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.histogram.as_mut().unwrap().reset();
                }
            }
        }
    }
}
