use super::ops::{
    basic_statistics::ComputeBasicStatisticsImpl,
    collect_data_for_editor::CollectDataForEditorImpl, color_mix::ColorMixImpl,
    contrast::AdjustContrastImpl, curve::ApplyCurveImpl, dehaze_apply::ApplyDehazeImpl,
    dehaze_prepare::PrepareDehazeImpl, exposure::AdjustExposureImpl,
    highlights_shadows::AdjustHighlightsAndShadowsImpl, histogram::ComputeHistogramImpl,
    temperature_tint::AdjustTemperatureAndTintImpl,
    vibrance_saturation::AdjustVibranceAndSaturationImpl, vignette::AdjustVignetteImpl, crop::CropImpl,
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
    pub collect_data_for_editor: Option<CollectDataForEditorImpl>,
    pub crop: Option<CropImpl>,
}

impl OpImplCollection {
    pub fn new() -> Self {
        OpImplCollection {
            ..Default::default()
        }
    }
}
