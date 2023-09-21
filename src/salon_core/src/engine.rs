use std::sync::Arc;

use crate::{runtime::Runtime, ops::exposure::ExposureOp};

pub struct Engine {
    pub runtime: Arc<Runtime>,
    exposure: ExposureOp,
}

impl Engine {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let exposure = ExposureOp::new(runtime.clone());
        Engine {
            runtime,
            exposure,
        }
    }
}