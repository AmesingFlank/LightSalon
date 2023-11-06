use super::ops::{
    basic_statistics::ComputeBasicStatisticsImpl,
    collect_data_for_editor::CollectDataForEditorImpl, color_mix::ColorMixImpl,
    contrast::AdjustContrastImpl, curve::ApplyCurveImpl, exposure::AdjustExposureImpl,
    highlights_shadows::AdjustHighlightsAndShadowsImpl, histogram::ComputeHistogramImpl,
    temperature_tint::AdjustTemperatureAndTintImpl,
    vibrance_saturation::AdjustVibranceAndSaturationImpl,
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
    pub basic_statistics: Option<ComputeBasicStatisticsImpl>,
    pub histogram: Option<ComputeHistogramImpl>,
    pub collect_data_for_editor: Option<CollectDataForEditorImpl>,
}

impl OpImplCollection {
    pub fn new() -> Self {
        OpImplCollection {
            ..Default::default()
        }
    }
}
