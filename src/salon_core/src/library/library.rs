use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::runtime::{ColorSpace, Image, Toolbox};
use crate::runtime::{ImageFormat, Runtime};

pub struct Library {
    images: Vec<Arc<Image>>,
    toolbox: Arc<Toolbox>,
}

impl Library {
    pub fn new(toolbox: Arc<Toolbox>) -> Self {
        Self {
            images: Vec::new(),
            toolbox,
        }
    }

    pub fn num_images(&self) -> usize {
        self.images.len() as usize
    }
    pub fn add_image(&mut self, image: Arc<Image>) -> usize {
        let image = self
            .toolbox
            .convert_image_format(image, ImageFormat::Rgba16Float);
        let image = self
            .toolbox
            .convert_color_space(image, ColorSpace::LinearRGB);
        self.toolbox.generate_mipmap(&image);
        self.images.push(image);
        self.images.len() - 1
    }
    pub fn get_image(&mut self, index: usize) -> Arc<Image> {
        self.images[index].clone()
    }
    pub fn get_thumbnail(&mut self, index: usize) -> Arc<Image> {
        self.get_image(index)
    }
}
