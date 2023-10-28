use super::ops::{exposure::AdjustExposureImpl,  histogram::ComputeHistogramImpl, collect_data_for_editor::CollectDataForEditorImpl, contrast::AdjustContrastImpl, basic_statistics::ComputeBasicStatisticsImpl, highlights_shadows::AdjustHighlightsAndShadowsImpl, vibrance_saturation::AdjustVibranceAndSaturationImpl, temperature_tint::AdjustTemperatureAndTintImpl, curve::ApplyCurveImpl};

#[derive(Default)]
pub struct OpImplCollection {
    pub exposure: Option<AdjustExposureImpl>,
    pub contrast: Option<AdjustContrastImpl>,
    pub highlights_shadows: Option<AdjustHighlightsAndShadowsImpl>,
    pub temperature_tint: Option<AdjustTemperatureAndTintImpl>,
    pub vibrance_saturation: Option<AdjustVibranceAndSaturationImpl>,
    pub curve: Option<ApplyCurveImpl>,
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
