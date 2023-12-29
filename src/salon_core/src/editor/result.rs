use std::sync::Arc;

use crate::{buffer::Buffer, engine::common::ImageHistogram, image::Image, runtime::Runtime};

pub struct EditResult {
    pub final_image: Arc<Image>,
    pub histogram_final: ImageHistogram,
    pub masks: Vec<Arc<Image>>,
}
