use std::sync::Arc;

use crate::{buffer::Buffer, image::Image, runtime::Runtime, engine::common::ImageHistogram};

pub struct DataForEditor {
    pub histogram_final: ImageHistogram,
}

impl DataForEditor {
    pub fn from_buffer(buffer: &Buffer, runtime: &Runtime) -> Self {
        let buffer_ints: Vec<u32> = runtime.read_buffer(buffer);

        let r = buffer_ints.as_slice()
            [ImageHistogram::max_bins() * 0..ImageHistogram::max_bins() * 1]
            .to_vec();
        let g = buffer_ints.as_slice()
            [ImageHistogram::max_bins() * 1..ImageHistogram::max_bins() * 2]
            .to_vec();
        let b = buffer_ints.as_slice()
            [ImageHistogram::max_bins() * 2..ImageHistogram::max_bins() * 3]
            .to_vec();
        let luma = buffer_ints.as_slice()
            [ImageHistogram::max_bins() * 3..ImageHistogram::max_bins() * 4]
            .to_vec();

        let num_bins = buffer_ints[ImageHistogram::max_bins() * 4];

        let histogram_final = ImageHistogram {
            r,
            g,
            b,
            luma,
            num_bins,
        };
        DataForEditor { histogram_final }
    }
}

pub struct EditResult {
    pub final_image: Option<Arc<Image>>,
    pub data_for_editor: Option<DataForEditor>,
}

impl EditResult {
    pub fn new_empty() -> Self {
        EditResult {
            final_image: None,
            data_for_editor: None,
        }
    }
}
