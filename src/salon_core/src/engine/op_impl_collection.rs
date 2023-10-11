use super::ops::{exposure::ExposureAdjustImpl, saturation::SaturationAdjustImpl};

#[derive(Default)]
pub struct OpImplCollection {
    pub exposure: Option<ExposureAdjustImpl>,
    pub saturation: Option<SaturationAdjustImpl>,
}

impl OpImplCollection {
    pub fn new() -> Self {
        OpImplCollection {
            ..Default::default()
        }
    }
}
