use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::runtime::Runtime;
use crate::runtime::{ColorSpace, Image};

use crate::runtime::{ColorSpaceConverter, MipmapGenerator};

pub struct Library {
    images: Vec<Arc<Image>>,
    runtime: Arc<Runtime>,
    color_space_converter: ColorSpaceConverter,
    mipmap_generator: MipmapGenerator,
}

impl Library {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let color_space_converter = ColorSpaceConverter::new(runtime.clone());
        let mipmap_generator = MipmapGenerator::new(runtime.clone());
        Self {
            images: Vec::new(),
            runtime,
            color_space_converter,
            mipmap_generator,
        }
    }

    pub fn num_images(&self) -> usize {
        self.images.len() as usize
    }
    pub fn add_image(&mut self, image: Arc<Image>) -> usize {
        let img = self
            .color_space_converter
            .convert(image, ColorSpace::LinearRGB);
        self.mipmap_generator.generate(&img);
        self.images.push(img);
        self.images.len() - 1
    }
    pub fn get_image(&mut self, index: usize) -> Arc<Image> {
        self.images[index].clone()
    }
    pub fn get_thumbnail(&mut self, index: usize) -> Arc<Image> {
        self.get_image(index)
    }
}
