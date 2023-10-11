use super::ops::{exposure::AdjustExposureImpl, saturation::AdjustSaturationImpl};

#[derive(Default)]
pub struct OpImplCollection {
    pub exposure: Option<AdjustExposureImpl>,
    pub saturation: Option<AdjustSaturationImpl>,
}

impl OpImplCollection {
    pub fn new() -> Self {
        OpImplCollection {
            ..Default::default()
        }
    }
}
