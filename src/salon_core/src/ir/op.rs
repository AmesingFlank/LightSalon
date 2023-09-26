use super::Id;

#[derive(Clone)]
pub enum Op {
    Input(Input),
    ExposureAdjust(Id, Id)
}

#[derive(Clone)]
pub struct Input {
    pub result: Id
}

#[derive(Clone)]
pub struct ExposureAdjust {
    pub result: Id,
    pub arg: Id,
}

