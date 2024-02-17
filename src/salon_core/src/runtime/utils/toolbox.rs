use std::{
    borrow::BorrowMut,
    cell::RefCell,
    sync::{Arc, RwLock},
};

use crate::runtime::{ColorSpace, Image, ImageFormat, Runtime};

use super::{
    color_space_converter::ColorSpaceConverter, image_format_converter::ImageFormatConverter,
    mipmap_generator::MipmapGenerator,
};

pub struct Toolbox {
    mipmap_generator: RefCell<MipmapGenerator>,
    color_space_converter: RefCell<ColorSpaceConverter>,
    image_format_converter: RefCell<ImageFormatConverter>,
}

impl Toolbox {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Self {
            mipmap_generator: RefCell::new(MipmapGenerator::new(runtime.clone())),
            color_space_converter: RefCell::new(ColorSpaceConverter::new(runtime.clone())),
            image_format_converter: RefCell::new(ImageFormatConverter::new(runtime.clone())),
        }
    }

    pub fn encode_mipmap_generation_command(
        &self,
        img: &Image,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut gen = self.mipmap_generator.borrow_mut();
        gen.encode_mipmap_generation_command(img, encoder);
    }

    pub fn generate_mipmap(&self, img: &Image) {
        let mut gen = self.mipmap_generator.borrow_mut();
        gen.generate(img);
    }

    pub fn convert_image_format(
        &self,
        img: Arc<Image>,
        dest_image_format: ImageFormat,
    ) -> Arc<Image> {
        let mut converter = self.image_format_converter.borrow_mut();
        converter.convert(img, dest_image_format)
    }

    pub fn convert_color_space(
        &self,
        input_img: Arc<Image>,
        dest_color_space: ColorSpace,
    ) -> Arc<Image> {
        let mut converter = self.color_space_converter.borrow_mut();
        converter.convert(input_img, dest_color_space)
    }
}
