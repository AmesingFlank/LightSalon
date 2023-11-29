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
        collect_data_for_editor::CollectDataForEditorImpl,
        color_mix::ColorMixImpl,
        contrast::AdjustContrastImpl,
        crop::CropImpl,
        curve::ApplyCurveImpl,
        dehaze_apply::ApplyDehazeImpl,
        dehaze_prepare::PrepareDehazeImpl,
        exposure::AdjustExposureImpl,
        highlights_shadows::AdjustHighlightsAndShadowsImpl,
        histogram::{self, ComputeHistogramImpl},
        temperature_tint::AdjustTemperatureAndTintImpl,
        vibrance_saturation::AdjustVibranceAndSaturationImpl,
        vignette::AdjustVignetteImpl,
    },
    result::ProcessResult,
    value_store::ValueStore,
    DataForEditor, ExecutionContext,
};

pub struct Engine {
    pub runtime: Arc<Runtime>,
    pub op_impls: OpImplCollection,
}

impl Engine {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Engine {
            runtime,
            op_impls: OpImplCollection::new(),
        }
    }

    pub fn execute_module(
        &mut self,
        module: &Module,
        input_img: Arc<Image>,
        execution_context: &mut ExecutionContext,
    ) -> ProcessResult {
        let mut result = ProcessResult::new_empty();
        self.reset_op_impls(module);
        self.apply_ops(module, input_img, execution_context);

        let output_id = module.get_output_id().expect("expecting an output id");
        let output_value = execution_context
            .value_store
            .map
            .get(&output_id)
            .expect("cannot find output");
        let output_image = output_value.as_image().clone();

        if let Some(editor_data_id) = module.get_tagged_id(IdTag::DataForEditor) {
            let editor_data_buffer = execution_context
                .value_store
                .map
                .get(&editor_data_id)
                .expect("cannot find stats")
                .as_buffer();
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
            result.data_for_editor = Some(data_for_editor)
        }

        result.final_image = Some(output_image);
        result
    }

    fn apply_ops(
        &mut self,
        module: &Module,
        input_img: Arc<Image>,
        execution_context: &mut ExecutionContext,
    ) {
        let ops = module.ops();
        let mut encoder = self
            .runtime
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let has_prev_execution = execution_context.last_input_image_uuid.is_some()
            && execution_context.last_module.is_some();
        let mut can_reuse_prev_value =
            has_prev_execution && execution_context.last_input_image_uuid == Some(input_img.uuid);

        for i in 0..ops.len() {
            let op = &ops[i];

            if can_reuse_prev_value {
                let last_module = execution_context.last_module.as_ref().unwrap();
                can_reuse_prev_value = i < last_module.ops().len() && *op == last_module.ops()[i];
            }

            if can_reuse_prev_value {
                continue;
            }

            match op {
                Op::Input(ref input) => {
                    execution_context
                        .value_store
                        .map
                        .insert(input.result, Value::Image(input_img.clone()));
                }
                Op::AdjustExposure(ref op) => {
                    self.op_impls.exposure.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut execution_context.value_store,
                    );
                }
                Op::AdjustContrast(ref op) => {
                    self.op_impls.contrast.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut execution_context.value_store,
                    );
                }
                Op::AdjustHighlightsAndShadows(ref op) => {
                    self.op_impls
                        .highlights_shadows
                        .as_mut()
                        .unwrap()
                        .encode_commands(&mut encoder, op, &mut execution_context.value_store);
                }
                Op::ApplyCurve(ref op) => {
                    self.op_impls.curve.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut execution_context.value_store,
                    );
                }
                Op::AdjustTemperatureAndTint(ref op) => {
                    self.op_impls
                        .temperature_tint
                        .as_mut()
                        .unwrap()
                        .encode_commands(&mut encoder, op, &mut execution_context.value_store);
                }
                Op::AdjustVibranceAndSaturation(ref op) => {
                    self.op_impls
                        .vibrance_saturation
                        .as_mut()
                        .unwrap()
                        .encode_commands(&mut encoder, op, &mut execution_context.value_store);
                }
                Op::ColorMix(ref op) => {
                    self.op_impls.color_mix.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut execution_context.value_store,
                    );
                }
                Op::AdjustVignette(ref op) => {
                    self.op_impls.vignette.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut execution_context.value_store,
                    );
                }
                Op::PrepareDehaze(ref op) => {
                    self.op_impls
                        .prepare_dehaze
                        .as_mut()
                        .unwrap()
                        .encode_commands(&mut encoder, op, &mut execution_context.value_store);
                }
                Op::ApplyDehaze(ref op) => {
                    self.op_impls
                        .apply_dehaze
                        .as_mut()
                        .unwrap()
                        .encode_commands(&mut encoder, op, &mut execution_context.value_store);
                }
                Op::ComputeBasicStatistics(ref op) => {
                    self.op_impls
                        .basic_statistics
                        .as_mut()
                        .unwrap()
                        .encode_commands(&mut encoder, op, &mut execution_context.value_store);
                }
                Op::ComputeHistogram(ref op) => {
                    self.op_impls.histogram.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut execution_context.value_store,
                    );
                }
                Op::CollectDataForEditor(ref op) => {
                    self.op_impls
                        .collect_data_for_editor
                        .as_mut()
                        .unwrap()
                        .encode_commands(&mut encoder, op, &mut execution_context.value_store);
                }
                Op::Crop(ref op) => {
                    self.op_impls.crop.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut execution_context.value_store,
                    );
                }
            }
        }

        let output_id = module.get_output_id().expect("expecting an output id");
        let output_value = execution_context
            .value_store
            .map
            .get(&output_id)
            .expect("cannot find output");
        let output_image = output_value.as_image();
        self.runtime
            .encode_mipmap_generation_command(&output_image.as_ref(), &mut encoder);

        self.runtime.queue.submit(Some(encoder.finish()));

        execution_context.last_module = Some(module.clone());
        execution_context.last_input_image_uuid = Some(input_img.uuid);
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
                Op::AdjustHighlightsAndShadows(_) => {
                    if self.op_impls.highlights_shadows.is_none() {
                        self.op_impls.highlights_shadows =
                            Some(AdjustHighlightsAndShadowsImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.highlights_shadows.as_mut().unwrap().reset();
                }
                Op::ApplyCurve(_) => {
                    if self.op_impls.curve.is_none() {
                        self.op_impls.curve = Some(ApplyCurveImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.curve.as_mut().unwrap().reset();
                }
                Op::AdjustTemperatureAndTint(_) => {
                    if self.op_impls.temperature_tint.is_none() {
                        self.op_impls.temperature_tint =
                            Some(AdjustTemperatureAndTintImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.temperature_tint.as_mut().unwrap().reset();
                }
                Op::AdjustVibranceAndSaturation(_) => {
                    if self.op_impls.vibrance_saturation.is_none() {
                        self.op_impls.vibrance_saturation =
                            Some(AdjustVibranceAndSaturationImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.vibrance_saturation.as_mut().unwrap().reset();
                }
                Op::ColorMix(_) => {
                    if self.op_impls.color_mix.is_none() {
                        self.op_impls.color_mix = Some(ColorMixImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.color_mix.as_mut().unwrap().reset();
                }
                Op::AdjustVignette(_) => {
                    if self.op_impls.vignette.is_none() {
                        self.op_impls.vignette = Some(AdjustVignetteImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.vignette.as_mut().unwrap().reset();
                }
                Op::PrepareDehaze(_) => {
                    if self.op_impls.prepare_dehaze.is_none() {
                        self.op_impls.prepare_dehaze =
                            Some(PrepareDehazeImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.prepare_dehaze.as_mut().unwrap().reset();
                }
                Op::ApplyDehaze(_) => {
                    if self.op_impls.apply_dehaze.is_none() {
                        self.op_impls.apply_dehaze =
                            Some(ApplyDehazeImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.apply_dehaze.as_mut().unwrap().reset();
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
                Op::CollectDataForEditor(_) => {
                    if self.op_impls.collect_data_for_editor.is_none() {
                        self.op_impls.collect_data_for_editor =
                            Some(CollectDataForEditorImpl::new(self.runtime.clone()))
                    }
                    self.op_impls
                        .collect_data_for_editor
                        .as_mut()
                        .unwrap()
                        .reset();
                }
                Op::Crop(_) => {
                    if self.op_impls.crop.is_none() {
                        self.op_impls.crop = Some(CropImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.crop.as_mut().unwrap().reset();
                }
            }
        }
    }
}
