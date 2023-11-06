use super::Id;

#[derive(Clone)]
pub enum Op {
    Input(InputOp),
    AdjustExposure(AdjustExposureOp),
    AdjustContrast(AdjustContrastOp),
    AdjustHighlightsAndShadows(AdjustHighlightsAndShadowsOp),
    ApplyCurve(ApplyCurveOp),
    AdjustTemperatureAndTint(AdjustTemperatureAndTintOp),
    AdjustVibranceAndSaturation(AdjustVibranceAndSaturationOp),
    ColorMix(ColorMixOp),
    ComputeBasicStatistics(ComputeBasicStatisticsOp),
    ComputeHistogram(ComputeHistogramOp),
    CollectDataForEditor(CollectDataForEditorOp),
}

#[derive(Clone)]
pub struct InputOp {
    pub result: Id,
}

#[derive(Clone)]
pub struct AdjustExposureOp {
    pub result: Id,
    pub arg: Id,
    pub exposure: f32,
}

#[derive(Clone)]
pub struct AdjustContrastOp {
    pub result: Id,
    pub arg: Id,
    pub basic_stats: Id,
    pub contrast: f32,
}

#[derive(Clone)]
pub struct AdjustHighlightsAndShadowsOp {
    pub result: Id,
    pub arg: Id,
    pub highlights: f32,
    pub shadows: f32,
}

#[derive(Clone)]
pub struct ApplyCurveOp {
    pub result: Id,
    pub arg: Id,
    pub control_points: Vec<(f32, f32)>,
    pub apply_r: bool,
    pub apply_g: bool,
    pub apply_b: bool,
}

// grouping temp and tint together, because they are heavy and shares a lot of common work
#[derive(Clone)]
pub struct AdjustTemperatureAndTintOp {
    pub result: Id,
    pub arg: Id,
    pub temperature: f32,
    pub tint: f32,
}

#[derive(Clone)]
pub struct AdjustVibranceAndSaturationOp {
    pub result: Id,
    pub arg: Id,
    pub vibrance: f32,
    pub saturation: f32,
}

#[derive(Clone)]
pub struct ColorMixOp {
    pub result: Id,
    pub arg: Id,
    pub hue_range: (f32, f32),
    pub groups: [ColorMixGroup; 8],
}

#[derive(Clone)]
pub struct ColorMixGroup {
    pub hue: f32,
    pub saturation: f32,
    pub lightness: f32,
}

#[derive(Clone)]
pub struct ComputeBasicStatisticsOp {
    pub result: Id,
    pub arg: Id,
}

#[derive(Clone)]
pub struct ComputeHistogramOp {
    pub result: Id,
    pub arg: Id,
}

#[derive(Clone)]
pub struct CollectDataForEditorOp {
    pub result: Id,
    pub histogram_final: Id,
}
