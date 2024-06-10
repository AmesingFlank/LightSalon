use crate::utils::rectangle::Rectangle;

use super::{GlobalMask, Id, LinearGradientMask, Mask, RadialGradientMask};

#[derive(Clone, PartialEq, Debug)]
pub enum Op {
    Input(InputOp),
    AdjustExposure(AdjustExposureOp),
    AdjustContrast(AdjustContrastOp),
    AdjustHighlightsAndShadows(AdjustHighlightsAndShadowsOp),
    ApplyCurve(ApplyCurveOp),
    AdjustTemperatureAndTint(AdjustTemperatureAndTintOp),
    AdjustVibranceAndSaturation(AdjustVibranceAndSaturationOp),
    ColorMix(ColorMixOp),
    AdjustVignette(AdjustVignetteOp),
    PrepareDehaze(PrepareDehazeOp),
    ApplyDehaze(ApplyDehazeOp),
    ComputeBasicStatistics(ComputeBasicStatisticsOp),
    ComputeHistogram(ComputeHistogramOp),
    RotateAndCrop(RotateAndCropOp),
    Resize(ResizeOp),
    ComputeGlobalMask(ComputeGlobalMaskOp),
    ComputeRadialGradientMask(ComputeRadialGradientMaskOp),
    ComputeLinearGradientMask(ComputeLinearGradientMaskOp),
    AddMask(AddMaskOp),
    SubtractMask(SubtractMaskOp),
    InvertMask(InvertMaskOp),
    ApplyMaskedEdits(ApplyMaskedEditsOp),
    ApplyFraming(ApplyFramingOp),
}

impl Op {
    pub fn get_arg_ids(&self) -> Vec<Id> {
        match self {
            Op::Input(ref o) => vec![],
            Op::AdjustExposure(ref o) => vec![o.arg],
            Op::AdjustContrast(ref o) => vec![o.arg, o.basic_stats],
            Op::AdjustHighlightsAndShadows(ref o) => vec![o.arg],
            Op::ApplyCurve(ref o) => vec![o.arg],
            Op::AdjustTemperatureAndTint(ref o) => vec![o.arg],
            Op::AdjustVibranceAndSaturation(ref o) => vec![o.arg],
            Op::ColorMix(ref o) => vec![o.arg],
            Op::PrepareDehaze(ref o) => vec![o.arg],
            Op::AdjustVignette(ref o) => vec![o.arg],
            Op::ApplyDehaze(ref o) => vec![o.arg],
            Op::ComputeBasicStatistics(ref o) => vec![o.arg],
            Op::ComputeHistogram(ref o) => vec![o.arg],
            Op::RotateAndCrop(ref o) => vec![o.arg],
            Op::Resize(ref o) => vec![o.arg],
            Op::ComputeGlobalMask(ref o) => vec![o.target],
            Op::ComputeRadialGradientMask(ref o) => vec![o.target],
            Op::ComputeLinearGradientMask(ref o) => vec![o.target],
            Op::AddMask(ref o) => vec![o.mask_0, o.mask_1],
            Op::SubtractMask(ref o) => vec![o.mask_0, o.mask_1],
            Op::InvertMask(ref o) => vec![o.mask_0],
            Op::ApplyMaskedEdits(ref o) => vec![o.original_target, o.edited, o.mask],
            Op::ApplyFraming(ref o) => vec![o.arg],
        }
    }

    pub fn get_result_id(&self) -> Id {
        match self {
            Op::Input(ref o) => o.result,
            Op::AdjustExposure(ref o) => o.result,
            Op::AdjustContrast(ref o) => o.result,
            Op::AdjustHighlightsAndShadows(ref o) => o.result,
            Op::ApplyCurve(ref o) => o.result,
            Op::AdjustTemperatureAndTint(ref o) => o.result,
            Op::AdjustVibranceAndSaturation(ref o) => o.result,
            Op::ColorMix(ref o) => o.result,
            Op::PrepareDehaze(ref o) => o.result,
            Op::AdjustVignette(ref o) => o.result,
            Op::ApplyDehaze(ref o) => o.result,
            Op::ComputeBasicStatistics(ref o) => o.result,
            Op::ComputeHistogram(ref o) => o.result,
            Op::RotateAndCrop(ref o) => o.result,
            Op::Resize(ref o) => o.result,
            Op::ComputeGlobalMask(ref o) => o.result,
            Op::ComputeRadialGradientMask(ref o) => o.result,
            Op::ComputeLinearGradientMask(ref o) => o.result,
            Op::AddMask(ref o) => o.result,
            Op::SubtractMask(ref o) => o.result,
            Op::InvertMask(ref o) => o.result,
            Op::ApplyMaskedEdits(ref o) => o.result,
            Op::ApplyFraming(ref o) => o.result,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct InputOp {
    pub result: Id,
}

#[derive(Clone, PartialEq, Debug)]
pub struct AdjustExposureOp {
    pub result: Id,
    pub arg: Id,
    pub exposure: f32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct AdjustContrastOp {
    pub result: Id,
    pub arg: Id,
    pub basic_stats: Id,
    pub contrast: f32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct AdjustHighlightsAndShadowsOp {
    pub result: Id,
    pub arg: Id,
    pub highlights: f32,
    pub shadows: f32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ApplyCurveOp {
    pub result: Id,
    pub arg: Id,
    pub control_points: Vec<(f32, f32)>,
    pub apply_r: bool,
    pub apply_g: bool,
    pub apply_b: bool,
}

// grouping temp and tint together, because they are heavy and shares a lot of common work
#[derive(Clone, PartialEq, Debug)]
pub struct AdjustTemperatureAndTintOp {
    pub result: Id,
    pub arg: Id,
    pub temperature: f32,
    pub tint: f32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct AdjustVibranceAndSaturationOp {
    pub result: Id,
    pub arg: Id,
    pub vibrance: f32,
    pub saturation: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ColorMixOp {
    pub result: Id,
    pub arg: Id,
    pub groups: [ColorMixGroup; 8],
}

#[derive(Clone, Copy, PartialEq, Debug, serde::Deserialize, serde::Serialize)]
pub struct ColorMixGroup {
    pub hue: f32,
    pub saturation: f32,
    pub lightness: f32,
}

impl ColorMixGroup {
    pub fn new() -> Self {
        Self {
            hue: 0.0,
            saturation: 0.0,
            lightness: 0.0,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct AdjustVignetteOp {
    pub result: Id,
    pub arg: Id,
    pub vignette: f32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct PrepareDehazeOp {
    pub result: Id,
    pub arg: Id,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ApplyDehazeOp {
    pub result: Id,
    pub arg: Id,
    pub dehazed: Id,
    pub amount: f32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ComputeBasicStatisticsOp {
    pub result: Id,
    pub arg: Id,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ComputeHistogramOp {
    pub result: Id,
    pub arg: Id,
}

#[derive(Clone, PartialEq, Debug)]
pub struct RotateAndCropOp {
    pub result: Id,
    pub arg: Id,
    pub rotation_degrees: f32,
    pub crop_rect: Rectangle,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ResizeOp {
    pub result: Id,
    pub arg: Id,
    pub factor: f32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ComputeGlobalMaskOp {
    pub result: Id,
    pub mask: GlobalMask,
    pub target: Id,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ComputeRadialGradientMaskOp {
    pub result: Id,
    pub mask: RadialGradientMask,
    pub target: Id,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ComputeLinearGradientMaskOp {
    pub result: Id,
    pub mask: LinearGradientMask,
    pub target: Id,
}

#[derive(Clone, PartialEq, Debug)]
pub struct AddMaskOp {
    pub result: Id,
    pub mask_0: Id,
    pub mask_1: Id,
}

#[derive(Clone, PartialEq, Debug)]
pub struct SubtractMaskOp {
    pub result: Id,
    pub mask_0: Id,
    pub mask_1: Id,
}

#[derive(Clone, PartialEq, Debug)]
pub struct InvertMaskOp {
    pub result: Id,
    pub mask_0: Id,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ApplyMaskedEditsOp {
    pub result: Id,
    pub mask: Id,
    pub original_target: Id,
    pub edited: Id,
}

#[derive(Clone, PartialEq, Debug, serde::Deserialize, serde::Serialize)]
pub struct Frame {
    pub aspect_ratio: (u32, u32),
    pub gap: f32,
}

impl Frame {
    pub fn defualt() -> Self {
        Self {
            aspect_ratio: (1, 1),
            gap: 0.1,
        }
    }

    pub fn aspect_ratio_float(&self) -> f32 {
        self.aspect_ratio.0 as f32 / self.aspect_ratio.1 as f32
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ApplyFramingOp {
    pub result: Id,
    pub arg: Id,
    pub frame: Frame,
}
