use std::{collections::HashMap, sync::Arc};

use crate::{
    ir::{Id, InputOp, Module, Op, Value},
    runtime::{Image, MipmapGenerator, Runtime},
};

use super::{
    op_impl_collection::OpImplCollection,
    ops::{
        apply_masked_edits::ApplyMaskedEditsImpl,
        basic_statistics::ComputeBasicStatisticsImpl,
        color_mix::ColorMixImpl,
        contrast::AdjustContrastImpl,
        crop::CropImpl,
        curve::ApplyCurveImpl,
        dehaze_apply::ApplyDehazeImpl,
        dehaze_prepare::PrepareDehazeImpl,
        exposure::AdjustExposureImpl,
        global_mask::ComputeGlobalMaskImpl,
        highlights_shadows::AdjustHighlightsAndShadowsImpl,
        histogram::{self, ComputeHistogramImpl},
        temperature_tint::AdjustTemperatureAndTintImpl,
        vibrance_saturation::AdjustVibranceAndSaturationImpl,
        vignette::AdjustVignetteImpl,
    },
    toolbox::Toolbox,
    value_store::ValueStore,
    ExecutionContext,
};

pub struct Engine {
    runtime: Arc<Runtime>,
    op_impls: OpImplCollection,
    toolbox: Toolbox,
}

impl Engine {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let toolbox = Toolbox::new(runtime.clone());
        Engine {
            runtime,
            toolbox,
            op_impls: OpImplCollection::new(),
        }
    }

    pub fn execute_module(
        &mut self,
        module: &Module,
        input_img: Arc<Image>,
        execution_context: &mut ExecutionContext,
    ) {
        self.reset_op_impls(module);
        self.apply_ops(module, input_img, execution_context);
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

        let reusable_ids_set = execution_context.compute_reusable_ids_set(module, input_img.uuid);

        for i in 0..ops.len() {
            let op = &ops[i];

            let result_id = op.get_result_id();
            if reusable_ids_set.contains(&result_id) {
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
                        &mut self.toolbox,
                    );
                }
                Op::AdjustContrast(ref op) => {
                    self.op_impls.contrast.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut execution_context.value_store,
                        &mut self.toolbox,
                    );
                }
                Op::AdjustHighlightsAndShadows(ref op) => {
                    self.op_impls
                        .highlights_shadows
                        .as_mut()
                        .unwrap()
                        .encode_commands(
                            &mut encoder,
                            op,
                            &mut execution_context.value_store,
                            &mut self.toolbox,
                        );
                }
                Op::ApplyCurve(ref op) => {
                    self.op_impls.curve.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut execution_context.value_store,
                        &mut self.toolbox,
                    );
                }
                Op::AdjustTemperatureAndTint(ref op) => {
                    self.op_impls
                        .temperature_tint
                        .as_mut()
                        .unwrap()
                        .encode_commands(
                            &mut encoder,
                            op,
                            &mut execution_context.value_store,
                            &mut self.toolbox,
                        );
                }
                Op::AdjustVibranceAndSaturation(ref op) => {
                    self.op_impls
                        .vibrance_saturation
                        .as_mut()
                        .unwrap()
                        .encode_commands(
                            &mut encoder,
                            op,
                            &mut execution_context.value_store,
                            &mut self.toolbox,
                        );
                }
                Op::ColorMix(ref op) => {
                    self.op_impls.color_mix.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut execution_context.value_store,
                        &mut self.toolbox,
                    );
                }
                Op::AdjustVignette(ref op) => {
                    self.op_impls.vignette.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut execution_context.value_store,
                        &mut self.toolbox,
                    );
                }
                Op::PrepareDehaze(ref op) => {
                    self.op_impls
                        .prepare_dehaze
                        .as_mut()
                        .unwrap()
                        .encode_commands(
                            &mut encoder,
                            op,
                            &mut execution_context.value_store,
                            &mut self.toolbox,
                        );
                }
                Op::ApplyDehaze(ref op) => {
                    self.op_impls
                        .apply_dehaze
                        .as_mut()
                        .unwrap()
                        .encode_commands(
                            &mut encoder,
                            op,
                            &mut execution_context.value_store,
                            &mut self.toolbox,
                        );
                }
                Op::ComputeBasicStatistics(ref op) => {
                    self.op_impls
                        .basic_statistics
                        .as_mut()
                        .unwrap()
                        .encode_commands(
                            &mut encoder,
                            op,
                            &mut execution_context.value_store,
                            &mut self.toolbox,
                        );
                }
                Op::ComputeHistogram(ref op) => {
                    self.op_impls.histogram.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut execution_context.value_store,
                        &mut self.toolbox,
                    );
                }
                Op::Crop(ref op) => {
                    self.op_impls.crop.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut execution_context.value_store,
                        &mut self.toolbox,
                    );
                }
                Op::ComputeGlobalMask(ref op) => {
                    self.op_impls.global_mask.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut execution_context.value_store,
                        &mut self.toolbox,
                    );
                }
                Op::ApplyMaskedEdits(ref op) => {
                    self.op_impls
                        .apply_masked_edits
                        .as_mut()
                        .unwrap()
                        .encode_commands(
                            &mut encoder,
                            op,
                            &mut execution_context.value_store,
                            &mut self.toolbox,
                        );
                }
            }
        }

        self.runtime.queue.submit(Some(encoder.finish()));

        execution_context.set_last(module.clone(), input_img.uuid);
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
                Op::Crop(_) => {
                    if self.op_impls.crop.is_none() {
                        self.op_impls.crop = Some(CropImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.crop.as_mut().unwrap().reset();
                }
                Op::ComputeGlobalMask(_) => {
                    if self.op_impls.global_mask.is_none() {
                        self.op_impls.global_mask =
                            Some(ComputeGlobalMaskImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.global_mask.as_mut().unwrap().reset();
                }
                Op::ApplyMaskedEdits(_) => {
                    if self.op_impls.apply_masked_edits.is_none() {
                        self.op_impls.apply_masked_edits =
                            Some(ApplyMaskedEditsImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.apply_masked_edits.as_mut().unwrap().reset();
                }
            }
        }
    }
}
