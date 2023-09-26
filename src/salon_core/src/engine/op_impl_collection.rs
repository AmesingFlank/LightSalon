use super::ops::exposure::ExposureAdjustImpl;

#[derive(Default)]
pub struct OpImplCollection {
    pub exposure: Option<ExposureAdjustImpl>,
}

impl OpImplCollection {
    pub fn new() -> Self {
        OpImplCollection {
            ..Default::default()
        }
    }
}
