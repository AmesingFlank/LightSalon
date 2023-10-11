use super::Id;

#[derive(Clone)]
pub enum Op {
    Input(Input),
    AdjustExposure(AdjustExposureOp),
    AdjustSaturation(AdjustSaturationOp),
}

#[derive(Clone)]
pub struct Input {
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
