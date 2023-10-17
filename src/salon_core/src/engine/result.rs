use std::sync::Arc;

use crate::{buffer::Buffer, image::Image, runtime::Runtime};

pub struct ImageHistogram {
    pub r: Vec<u32>,
    pub g: Vec<u32>,
    pub b: Vec<u32>,
    pub luma: Vec<u32>,
}

pub struct ImageStatistics {
    pub histogram_final: ImageHistogram,
}

impl ImageStatistics {
    pub fn from_buffer(buffer: &Buffer, runtime: &Runtime) -> Self {
        let buffer_ints: Vec<u32> = runtime.read_buffer(buffer);

        let r = buffer_ints.as_slice()[256 * 0..256 * 1].to_vec();
        let g = buffer_ints.as_slice()[256 * 1..256 * 2].to_vec();
        let b = buffer_ints.as_slice()[256 * 2..256 * 3].to_vec();
        let luma = buffer_ints.as_slice()[256 * 3..256 * 4].to_vec();

        let histogram_final = ImageHistogram { r, g, b, luma };
        ImageStatistics { histogram_final }
    }
}

pub struct ProcessResult {
    pub final_image: Option<Arc<Image>>,
    pub statistics: Option<ImageStatistics>,
}

impl ProcessResult {
    pub fn new_empty() -> Self {
        ProcessResult {
            final_image: None,
            statistics: None,
        }
    }
}
