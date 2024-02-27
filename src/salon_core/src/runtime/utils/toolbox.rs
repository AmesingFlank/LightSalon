use std::sync::{Arc, RwLock};

use crate::runtime::{Buffer, ColorSpace, Image, ImageFormat, Runtime};

use super::{
    color_space_converter::ColorSpaceConverter, image_format_converter::ImageFormatConverter,
    image_resizer::ImageResizer, image_to_buffer_copier::ImageToBufferCopier,
    mipmap_generator::MipmapGenerator,
};

pub struct Toolbox {
    mipmap_generator: RwLock<MipmapGenerator>,
    color_space_converter: RwLock<ColorSpaceConverter>,
    image_format_converter: RwLock<ImageFormatConverter>,
    image_resizer: RwLock<ImageResizer>,
    image_to_buffer_copier: RwLock<ImageToBufferCopier>,
}

impl Toolbox {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Self {
            mipmap_generator: RwLock::new(MipmapGenerator::new(runtime.clone())),
            color_space_converter: RwLock::new(ColorSpaceConverter::new(runtime.clone())),
            image_format_converter: RwLock::new(ImageFormatConverter::new(runtime.clone())),
            image_to_buffer_copier: RwLock::new(ImageToBufferCopier::new(runtime.clone())),
            image_resizer: RwLock::new(ImageResizer::new(runtime.clone())),
        }
    }

    pub fn encode_mipmap_generation_command(
        &self,
        img: &Image,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut gen = self.mipmap_generator.write().unwrap();
        gen.encode_mipmap_generation_command(img, encoder);
    }

    pub fn generate_mipmap(&self, img: &Image) {
        let mut gen = self.mipmap_generator.write().unwrap();
        gen.generate(img);
    }

    pub fn convert_image_format(
        &self,
        img: Arc<Image>,
        dest_image_format: ImageFormat,
    ) -> Arc<Image> {
        let mut converter = self.image_format_converter.write().unwrap();
        converter.convert(img, dest_image_format)
    }

    pub fn convert_color_space(
        &self,
        input_img: Arc<Image>,
        dest_color_space: ColorSpace,
    ) -> Arc<Image> {
        let mut converter = self.color_space_converter.write().unwrap();
        converter.convert(input_img, dest_color_space)
    }

    pub fn resize_image(&self, input_img: Arc<Image>, factor: f32) -> Arc<Image> {
        let mut resizer = self.image_resizer.write().unwrap();
        resizer.resize(input_img, factor)
    }

    pub fn copy_image_to_buffer(&self, input_img: &Image) -> Arc<Buffer> {
        let mut copier = self.image_to_buffer_copier.write().unwrap();
        copier.copy(input_img)
    }
}
