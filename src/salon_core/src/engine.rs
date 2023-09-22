use std::sync::Arc;

use crate::{runtime::Runtime, ops::exposure::ExposureOp};

pub struct Engine {
    pub runtime: Arc<Runtime>,
    pub exposure_op: ExposureOp,
}

impl Engine {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let exposure_op = ExposureOp::new(runtime.clone());
        Engine {
            runtime,
            exposure_op,
        }
    }
}