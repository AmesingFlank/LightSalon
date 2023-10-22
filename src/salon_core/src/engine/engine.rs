use std::{collections::HashMap, sync::Arc};

use crate::{
    engine::ImageHistogram,
    image::Image,
    ir::{Id, IdTag, InputOp, Module, Op, Value},
    runtime::Runtime,
};

use super::{
    op_impl_collection::OpImplCollection,
    ops::{
        basic_statistics::ComputeBasicStatisticsImpl,
        collect_statistics::CollectStatisticsImpl,
        contrast::AdjustContrastImpl,
        exposure::AdjustExposureImpl,
        histogram::{self, ComputeHistogramImpl},
        saturation::{self, AdjustSaturationImpl}, highlights::AdjustHighlightsImpl,
    },
    result::ProcessResult,
    value_store::ValueStore,
    ImageStatistics,
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
        self.apply_ops(module, input_img);

        let output_id = module.get_output_id().expect("expecting an output id");
        let output_value = self
            .value_store
            .map
            .get(&output_id)
            .expect("cannot find output");
        let output_image = output_value.as_image().clone();

        if let Some(statistics_id) = module.get_tagged_id(IdTag::Statistics) {
            let statistics_buffer = self
                .value_store
                .map
                .get(&statistics_id)
                .expect("cannot find stats")
                .as_buffer();
            let stats = ImageStatistics::from_buffer(&statistics_buffer, &self.runtime);
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
            result.statistics = Some(stats)
        }

        result.final_image = Some(output_image);
        result
    }

    fn apply_ops(&mut self, module: &Module, input_img: Arc<Image>) {
        let ops = module.ops();
        let mut encoder = self
            .runtime
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        for op in ops {
            match op {
                Op::Input(ref input) => {
                    self.value_store
                        .map
                        .insert(input.result, Value::Image(input_img.clone()));
                }
                Op::AdjustExposure(ref op) => {
                    self.op_impls.exposure.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut self.value_store,
                    );
                }
                Op::AdjustContrast(ref op) => {
                    self.op_impls.contrast.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut self.value_store,
                    );
                }
                Op::AdjustHighlights(ref op) => {
                    self.op_impls.highlights.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut self.value_store,
                    );
                }
                Op::AdjustSaturation(ref op) => {
                    self.op_impls.saturation.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut self.value_store,
                    );
                }
                Op::ComputeBasicStatistics(ref op) => {
                    self.op_impls.basic_statistics.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut self.value_store,
                    );
                }
                Op::ComputeHistogram(ref op) => {
                    self.op_impls.histogram.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut self.value_store,
                    );
                }
                Op::CollectStatistics(ref op) => {
                    self.op_impls
                        .collect_statistics
                        .as_mut()
                        .unwrap()
                        .encode_commands(&mut encoder, op, &mut self.value_store);
                }
            }
        }

        let output_id = module.get_output_id().expect("expecting an output id");
        let output_value = self
            .value_store
            .map
            .get(&output_id)
            .expect("cannot find output");
        let output_image = output_value.as_image();
        self.runtime
            .encode_mipmap_generation_command(&output_image.as_ref(), &mut encoder);

        self.runtime.queue.submit(Some(encoder.finish()));
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
                Op::AdjustContrast(_) => {
                    if self.op_impls.contrast.is_none() {
                        self.op_impls.contrast = Some(AdjustContrastImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.contrast.as_mut().unwrap().reset();
                }
                Op::AdjustHighlights(_) => {
                    if self.op_impls.highlights.is_none() {
                        self.op_impls.highlights = Some(AdjustHighlightsImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.highlights.as_mut().unwrap().reset();
                }
                Op::AdjustSaturation(_) => {
                    if self.op_impls.saturation.is_none() {
                        self.op_impls.saturation =
                            Some(AdjustSaturationImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.saturation.as_mut().unwrap().reset();
                }
                Op::ComputeBasicStatistics(_) => {
                    if self.op_impls.basic_statistics.is_none() {
                        self.op_impls.basic_statistics =
                            Some(ComputeBasicStatisticsImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.basic_statistics.as_mut().unwrap().reset();
                }
                Op::ComputeHistogram(_) => {
                    if self.op_impls.histogram.is_none() {
                        self.op_impls.histogram =
                            Some(ComputeHistogramImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.histogram.as_mut().unwrap().reset();
                }
                Op::CollectStatistics(_) => {
                    if self.op_impls.collect_statistics.is_none() {
                        self.op_impls.collect_statistics =
                            Some(CollectStatisticsImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.collect_statistics.as_mut().unwrap().reset();
                }
            }
        }
    }
}
