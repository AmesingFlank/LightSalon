use super::Id;

#[derive(Clone)]
pub enum Op {
    Input(InputOp),
    AdjustExposure(AdjustExposureOp),
    AdjustSaturation(AdjustSaturationOp),
    ComputeHistogram(ComputeHistogramOp),
    CollectStatistics(CollectStatisticsOp),
}

#[derive(Clone)]
pub struct InputOp {
    pub result: Id
}

#[derive(Clone)]
pub struct AdjustExposureOp {
    pub result: Id,
    pub arg: Id,
    pub exposure: f32,
}

#[derive(Clone)]
pub struct AdjustSaturationOp {
    pub result: Id,
    pub arg: Id,
    pub saturation: f32,
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
