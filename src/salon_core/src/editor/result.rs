use std::sync::Arc;

use crate::engine::common::ImageHistogram;
use crate::runtime::{Buffer, Image, Runtime};

pub struct EditResult {
    pub final_image: Arc<Image>,
    pub histogram_final: ImageHistogram,
    pub masks: Vec<Arc<Image>>,
}
