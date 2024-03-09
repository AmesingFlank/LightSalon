use super::ops::{
    add_mask::AddMaskImpl, apply_masked_edits::ApplyMaskedEditsImpl,
    basic_statistics::ComputeBasicStatisticsImpl, color_mix::ColorMixImpl,
    contrast::AdjustContrastImpl, rotate_and_crop::RotateAndCropImpl, curve::ApplyCurveImpl,
    dehaze_apply::ApplyDehazeImpl, dehaze_prepare::PrepareDehazeImpl, exposure::AdjustExposureImpl,
    global_mask::ComputeGlobalMaskImpl, highlights_shadows::AdjustHighlightsAndShadowsImpl,
    histogram::ComputeHistogramImpl, invert_mask::InvertMaskImpl,
    linear_gradient_mask::ComputeLinearGradientMaskImpl,
    radial_gradient_mask::ComputeRadialGradientMaskImpl, subtract_mask::SubtractMaskImpl,
    temperature_tint::AdjustTemperatureAndTintImpl,
    vibrance_saturation::AdjustVibranceAndSaturationImpl, vignette::AdjustVignetteImpl, resize::ResizeImpl,
};

#[derive(Default)]
pub struct OpImplCollection {
    pub exposure: Option<AdjustExposureImpl>,
    pub contrast: Option<AdjustContrastImpl>,
    pub highlights_shadows: Option<AdjustHighlightsAndShadowsImpl>,
    pub curve: Option<ApplyCurveImpl>,
    pub temperature_tint: Option<AdjustTemperatureAndTintImpl>,
    pub vibrance_saturation: Option<AdjustVibranceAndSaturationImpl>,
    pub color_mix: Option<ColorMixImpl>,
    pub vignette: Option<AdjustVignetteImpl>,
    pub prepare_dehaze: Option<PrepareDehazeImpl>,
    pub apply_dehaze: Option<ApplyDehazeImpl>,
    pub basic_statistics: Option<ComputeBasicStatisticsImpl>,
    pub histogram: Option<ComputeHistogramImpl>,
    pub rotate_and_crop: Option<RotateAndCropImpl>,
    pub resize: Option<ResizeImpl>,
    pub global_mask: Option<ComputeGlobalMaskImpl>,
    pub radial_gradient_mask: Option<ComputeRadialGradientMaskImpl>,
    pub linear_gradient_mask: Option<ComputeLinearGradientMaskImpl>,
    pub add_mask: Option<AddMaskImpl>,
    pub subtract_mask: Option<SubtractMaskImpl>,
    pub invert_mask: Option<InvertMaskImpl>,
    pub apply_masked_edits: Option<ApplyMaskedEditsImpl>,
}

impl OpImplCollection {
    pub fn new() -> Self {
        OpImplCollection {
            ..Default::default()
        }
    }
}
