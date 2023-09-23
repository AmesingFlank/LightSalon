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

