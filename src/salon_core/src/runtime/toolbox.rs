use std::sync::Arc;

use crate::image::{ColorSpace, Image};

use super::{
    color_space_converter::ColorSpaceConverter, mipmap_generator::MipmapGenerator, Runtime,
};

pub struct ToolBox {
    pub mipmap_generator: MipmapGenerator,
    pub color_space_converter: ColorSpaceConverter,
}

impl ToolBox {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Self {
            mipmap_generator: MipmapGenerator::new(runtime.clone()),
            color_space_converter: ColorSpaceConverter::new(runtime.clone()),
        }
    }

    pub fn ensure_mipmap(&mut self, image: &Image) {
        self.mipmap_generator.generate(image);
    }

    pub fn encode_mipmap_generation_command(
        &mut self,
        image: &Image,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        self.mipmap_generator
            .encode_mipmap_generation_command(image, encoder);
    }

    pub fn convert_color_space(
        &self,
        input_img: Arc<Image>,
        dest_color_space: ColorSpace,
    ) -> Arc<Image> {
        self.color_space_converter
            .convert(input_img, dest_color_space)
    }
}
