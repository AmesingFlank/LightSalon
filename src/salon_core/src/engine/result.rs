use std::sync::Arc;

use crate::{buffer::Buffer, image::Image};

pub struct ProcessResult {
    pub final_image: Option<Arc<Image>>,
    pub histogram: Option<Arc<Buffer>>,
}

impl ProcessResult {
    pub fn new_empty() -> Self {
        ProcessResult {
            final_image: None,
            histogram: None,
        }
    }
}
