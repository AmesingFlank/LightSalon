use std::sync::Arc;

use crate::{buffer::Buffer, image::Image, runtime::Runtime};

pub struct ImageHistogram {
    pub r: Vec<u32>,
    pub g: Vec<u32>,
    pub b: Vec<u32>,
    pub luma: Vec<u32>,
    pub num_bins: u32,
}

impl ImageHistogram {
    pub fn max_bins() -> usize {
        256
    }
    pub fn num_bins_for(dimensions: (u32, u32)) -> usize {
        // need more sophisticated logic here?
        100
    }
    pub fn max_value(&self) -> u32 {
        let mut max = 0u32;
        for r in self.r.iter() {
            max = max.max(*r);
        }
        for g in self.g.iter() {
            max = max.max(*g);
        }
        for b in self.b.iter() {
            max = max.max(*b);
        }
        for luma in self.luma.iter() {
            max = max.max(*luma);
        }
        max
    }
}

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

pub struct ProcessResult {
    pub final_image: Option<Arc<Image>>,
    pub data_for_editor: Option<DataForEditor>,
}

impl ProcessResult {
    pub fn new_empty() -> Self {
        ProcessResult {
            final_image: None,
            data_for_editor: None,
        }
    }
}
