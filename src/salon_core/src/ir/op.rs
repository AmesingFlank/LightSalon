use super::Id;

#[derive(Clone, PartialEq)]
pub enum Op {
    Input(InputOp),
    AdjustExposure(AdjustExposureOp),
    AdjustContrast(AdjustContrastOp),
    AdjustHighlightsAndShadows(AdjustHighlightsAndShadowsOp),
    ApplyCurve(ApplyCurveOp),
    AdjustTemperatureAndTint(AdjustTemperatureAndTintOp),
    AdjustVibranceAndSaturation(AdjustVibranceAndSaturationOp),
    ColorMix(ColorMixOp),
    Dehaze(DehazeOp),
    ComputeBasicStatistics(ComputeBasicStatisticsOp),
    ComputeHistogram(ComputeHistogramOp),
    CollectDataForEditor(CollectDataForEditorOp),
}

#[derive(Clone, PartialEq)]
pub struct InputOp {
    pub result: Id,
}

#[derive(Clone, PartialEq)]
pub struct AdjustExposureOp {
    pub result: Id,
    pub arg: Id,
    pub exposure: f32,
}

#[derive(Clone, PartialEq)]
pub struct AdjustContrastOp {
    pub result: Id,
    pub arg: Id,
    pub basic_stats: Id,
    pub contrast: f32,
}

#[derive(Clone, PartialEq)]
pub struct AdjustHighlightsAndShadowsOp {
    pub result: Id,
    pub arg: Id,
    pub highlights: f32,
    pub shadows: f32,
}

#[derive(Clone, PartialEq)]
pub struct ApplyCurveOp {
    pub result: Id,
    pub arg: Id,
    pub control_points: Vec<(f32, f32)>,
    pub apply_r: bool,
    pub apply_g: bool,
    pub apply_b: bool,
}

// grouping temp and tint together, because they are heavy and shares a lot of common work
#[derive(Clone, PartialEq)]
pub struct AdjustTemperatureAndTintOp {
    pub result: Id,
    pub arg: Id,
    pub temperature: f32,
    pub tint: f32,
}

#[derive(Clone, PartialEq)]
pub struct AdjustVibranceAndSaturationOp {
    pub result: Id,
    pub arg: Id,
    pub vibrance: f32,
    pub saturation: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub struct ColorMixOp {
    pub result: Id,
    pub arg: Id,
    pub groups: [ColorMixGroup; 8],
}

#[derive(Clone, Copy, PartialEq)]
pub struct ColorMixGroup {
    pub hue: f32,
    pub saturation: f32,
    pub lightness: f32,
}

#[derive(Clone, PartialEq)]
pub struct DehazeOp {
    pub result: Id,
    pub arg: Id,
    pub dehaze: f32,
}

#[derive(Clone, PartialEq)]
pub struct ComputeBasicStatisticsOp {
    pub result: Id,
    pub arg: Id,
}

#[derive(Clone, PartialEq)]
pub struct ComputeHistogramOp {
    pub result: Id,
    pub arg: Id,
}

#[derive(Clone, PartialEq)]
pub struct CollectDataForEditorOp {
    pub result: Id,
    pub histogram_final: Id,
}
