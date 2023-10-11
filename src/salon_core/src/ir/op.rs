use super::Id;

#[derive(Clone)]
pub enum Op {
    Input(Input),
    ExposureAdjust(ExposureAdjust),
    SaturationAdjust(SaturationAdjust),
}

#[derive(Clone)]
pub struct Input {
    pub result: Id
}

#[derive(Clone)]
pub struct ExposureAdjust {
    pub result: Id,
    pub arg: Id,
    pub exposure: f32,
}

#[derive(Clone)]
pub struct SaturationAdjust {
    pub result: Id,
    pub arg: Id,
    pub saturation: f32,
}
