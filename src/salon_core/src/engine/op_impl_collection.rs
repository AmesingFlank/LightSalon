use super::ops::{exposure::ExposureAdjustImpl, brightness::BrightnessAdjustImpl};

#[derive(Default)]
pub struct OpImplCollection {
    pub exposure: Option<ExposureAdjustImpl>,
    pub brightness: Option<BrightnessAdjustImpl>,
}

impl OpImplCollection {
    pub fn new() -> Self {
        OpImplCollection {
            ..Default::default()
        }
    }
}
