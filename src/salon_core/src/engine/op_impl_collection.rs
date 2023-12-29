use super::ops::{
    apply_masked_edits::ApplyMaskedEditsImpl, basic_statistics::ComputeBasicStatisticsImpl,
    color_mix::ColorMixImpl, contrast::AdjustContrastImpl, crop::CropImpl, curve::ApplyCurveImpl,
    dehaze_apply::ApplyDehazeImpl, dehaze_prepare::PrepareDehazeImpl, exposure::AdjustExposureImpl,
    global_mask::ComputeGlobalMaskImpl, highlights_shadows::AdjustHighlightsAndShadowsImpl,
    histogram::ComputeHistogramImpl, temperature_tint::AdjustTemperatureAndTintImpl,
    vibrance_saturation::AdjustVibranceAndSaturationImpl, vignette::AdjustVignetteImpl,
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
    pub crop: Option<CropImpl>,
    pub global_mask: Option<ComputeGlobalMaskImpl>,
    pub apply_masked_edits: Option<ApplyMaskedEditsImpl>,
}

impl OpImplCollection {
    pub fn new() -> Self {
        OpImplCollection {
            ..Default::default()
        }
    }
}
