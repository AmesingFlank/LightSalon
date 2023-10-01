use super::Id;

#[derive(Clone)]
pub enum Op {
    Input(Input),
    ExposureAdjust(ExposureAdjust),
    BrightnessAdjust(BrightnessAdjust),
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
pub struct BrightnessAdjust {
    pub result: Id,
    pub arg: Id,
    pub brightness: f32,
}
