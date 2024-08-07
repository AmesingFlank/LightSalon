use std::{sync::Arc};



use crate::{
    ir::{Module, Op, Value},
    runtime::{Image, Runtime, Toolbox},
};

use super::{
    op_impl_collection::OpImplCollection,
    ops::{
        add_mask::AddMaskImpl,
        apply_masked_edits::ApplyMaskedEditsImpl,
        basic_statistics::ComputeBasicStatisticsImpl,
        color_mix::ColorMixImpl,
        contrast::AdjustContrastImpl,
        curve::ApplyCurveImpl,
        dehaze_apply::ApplyDehazeImpl,
        dehaze_prepare::PrepareDehazeImpl,
        exposure::AdjustExposureImpl,
        framing::ApplyFramingImpl,
        global_mask::ComputeGlobalMaskImpl,
        highlights_shadows::AdjustHighlightsAndShadowsImpl,
        histogram::{ComputeHistogramImpl},
        invert_mask::InvertMaskImpl,
        linear_gradient_mask::ComputeLinearGradientMaskImpl,
        radial_gradient_mask::ComputeRadialGradientMaskImpl,
        resize::ResizeImpl,
        rotate_and_crop::RotateAndCropImpl,
        subtract_mask::{SubtractMaskImpl},
        temperature_tint::AdjustTemperatureAndTintImpl,
        vibrance_saturation::AdjustVibranceAndSaturationImpl,
        vignette::AdjustVignetteImpl,
    },
    ExecutionContext,
};

pub struct Engine {
    runtime: Arc<Runtime>,
    toolbox: Arc<Toolbox>,
    op_impls: OpImplCollection,
}

impl Engine {
    pub fn new(runtime: Arc<Runtime>, toolbox: Arc<Toolbox>) -> Self {
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
                Op::RotateAndCrop(ref op) => {
                    self.op_impls
                        .rotate_and_crop
                        .as_mut()
                        .unwrap()
                        .encode_commands(
                            &mut encoder,
                            op,
                            &mut execution_context.value_store,
                            &mut self.toolbox,
                        );
                }
                Op::Resize(ref op) => {
                    self.op_impls.resize.as_mut().unwrap().encode_commands(
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
                Op::ComputeRadialGradientMask(ref op) => {
                    self.op_impls
                        .radial_gradient_mask
                        .as_mut()
                        .unwrap()
                        .encode_commands(
                            &mut encoder,
                            op,
                            &mut execution_context.value_store,
                            &mut self.toolbox,
                        );
                }
                Op::ComputeLinearGradientMask(ref op) => {
                    self.op_impls
                        .linear_gradient_mask
                        .as_mut()
                        .unwrap()
                        .encode_commands(
                            &mut encoder,
                            op,
                            &mut execution_context.value_store,
                            &mut self.toolbox,
                        );
                }
                Op::AddMask(ref op) => {
                    self.op_impls.add_mask.as_mut().unwrap().encode_commands(
                        &mut encoder,
                        op,
                        &mut execution_context.value_store,
                        &mut self.toolbox,
                    );
                }
                Op::SubtractMask(ref op) => {
                    self.op_impls
                        .subtract_mask
                        .as_mut()
                        .unwrap()
                        .encode_commands(
                            &mut encoder,
                            op,
                            &mut execution_context.value_store,
                            &mut self.toolbox,
                        );
                }
                Op::InvertMask(ref op) => {
                    self.op_impls.invert_mask.as_mut().unwrap().encode_commands(
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
                Op::ApplyFraming(ref op) => {
                    self.op_impls.framing.as_mut().unwrap().encode_commands(
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
                Op::RotateAndCrop(_) => {
                    if self.op_impls.rotate_and_crop.is_none() {
                        self.op_impls.rotate_and_crop =
                            Some(RotateAndCropImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.rotate_and_crop.as_mut().unwrap().reset();
                }
                Op::Resize(_) => {
                    if self.op_impls.resize.is_none() {
                        self.op_impls.resize = Some(ResizeImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.resize.as_mut().unwrap().reset();
                }
                Op::ComputeGlobalMask(_) => {
                    if self.op_impls.global_mask.is_none() {
                        self.op_impls.global_mask =
                            Some(ComputeGlobalMaskImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.global_mask.as_mut().unwrap().reset();
                }
                Op::ComputeRadialGradientMask(_) => {
                    if self.op_impls.radial_gradient_mask.is_none() {
                        self.op_impls.radial_gradient_mask =
                            Some(ComputeRadialGradientMaskImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.radial_gradient_mask.as_mut().unwrap().reset();
                }
                Op::ComputeLinearGradientMask(_) => {
                    if self.op_impls.linear_gradient_mask.is_none() {
                        self.op_impls.linear_gradient_mask =
                            Some(ComputeLinearGradientMaskImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.linear_gradient_mask.as_mut().unwrap().reset();
                }
                Op::AddMask(_) => {
                    if self.op_impls.add_mask.is_none() {
                        self.op_impls.add_mask = Some(AddMaskImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.add_mask.as_mut().unwrap().reset();
                }
                Op::SubtractMask(_) => {
                    if self.op_impls.subtract_mask.is_none() {
                        self.op_impls.subtract_mask =
                            Some(SubtractMaskImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.subtract_mask.as_mut().unwrap().reset();
                }
                Op::InvertMask(_) => {
                    if self.op_impls.invert_mask.is_none() {
                        self.op_impls.invert_mask = Some(InvertMaskImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.invert_mask.as_mut().unwrap().reset();
                }
                Op::ApplyMaskedEdits(_) => {
                    if self.op_impls.apply_masked_edits.is_none() {
                        self.op_impls.apply_masked_edits =
                            Some(ApplyMaskedEditsImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.apply_masked_edits.as_mut().unwrap().reset();
                }
                Op::ApplyFraming(_) => {
                    if self.op_impls.framing.is_none() {
                        self.op_impls.framing = Some(ApplyFramingImpl::new(self.runtime.clone()))
                    }
                    self.op_impls.framing.as_mut().unwrap().reset();
                }
            }
        }
    }
}
