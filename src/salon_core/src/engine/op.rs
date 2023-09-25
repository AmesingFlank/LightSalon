use std::sync::Arc;

use crate::image::Image;

pub enum OpType {
    ExposureAdjust
}


pub trait Op {
    fn apply(
        &mut self,
        inputs: Vec<Arc<Image>>,
        outputs: Vec<Arc<Image>>,
        params: serde_json::Value
    );
}

pub trait SingleImageOp {
    fn apply(
        &mut self,
        input: Arc<Image>,
        output: Arc<Image>,
        params: serde_json::Value
    );
}

pub struct OpFromSingleImgeOp {
    op: Box<dyn SingleImageOp>
}

impl OpFromSingleImgeOp {
    pub fn new(op: Box<dyn SingleImageOp>) -> Self {
        OpFromSingleImgeOp {
            op
        }
    }
}
