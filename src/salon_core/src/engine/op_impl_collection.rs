use super::ops::{exposure::AdjustExposureImpl, saturation::AdjustSaturationImpl, histogram::ComputeHistogramImpl, collect_data_for_editor::CollectDataForEditorImpl, contrast::AdjustContrastImpl, basic_statistics::ComputeBasicStatisticsImpl, highlights::AdjustHighlightsImpl, shadows::AdjustShadowsImpl, vibrance::AdjustVibranceImpl, temperature_tint::AdjustTemperatureAndTintImpl};

#[derive(Default)]
pub struct OpImplCollection {
    pub exposure: Option<AdjustExposureImpl>,
    pub contrast: Option<AdjustContrastImpl>,
    pub highlights: Option<AdjustHighlightsImpl>,
    pub shadows: Option<AdjustShadowsImpl>,
    pub temperature_tint: Option<AdjustTemperatureAndTintImpl>,
    pub vibrance: Option<AdjustVibranceImpl>,
    pub saturation: Option<AdjustSaturationImpl>,
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
