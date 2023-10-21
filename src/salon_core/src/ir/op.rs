use super::Id;

#[derive(Clone)]
pub enum Op {
    Input(InputOp),
    AdjustExposure(AdjustExposureOp),
    AdjustContrast(AdjustContrastOp),
    AdjustSaturation(AdjustSaturationOp),
    ComputeBasicStatistics(ComputeBasicStatisticsOp),
    ComputeHistogram(ComputeHistogramOp),
    CollectStatistics(CollectStatisticsOp),
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
    pub contrast: f32,
}

#[derive(Clone)]
pub struct AdjustSaturationOp {
    pub result: Id,
    pub arg: Id,
    pub saturation: f32,
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
pub struct CollectStatisticsOp {
    pub result: Id,
    pub histogram: Id,
}
