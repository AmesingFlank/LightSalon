use image::GenericImageView;
use std::path::PathBuf;

use crate::runtime;

pub struct Image {
    pub properties: ImageProperties,
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub texture_view_base_mip: wgpu::TextureView,
    pub uuid: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    Linear = 0,
    sRGB = 1,
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Rgba16Float,
}

impl ImageFormat {
    pub fn to_wgpu_texture_format(&self) -> wgpu::TextureFormat {
        match *self {
            ImageFormat::Rgba16Float =>  wgpu::TextureFormat::Rgba16Float,
        }       
    }
    pub fn bytes_per_channel(&self) -> u32 {
        match *self {
            ImageFormat::Rgba16Float => 2,
        } 
    }
    pub fn bytes_per_pixel(&self) -> u32 {
        match *self {
            ImageFormat::Rgba16Float => 8,
        } 
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ImageProperties {
    pub dimensions: (u32, u32),
    pub format: ImageFormat,
    pub color_space: ColorSpace,
}

impl Image {
    pub fn aspect_ratio(&self) -> f32 {
        self.properties.dimensions.1 as f32 / self.properties.dimensions.0 as f32
    }
    pub fn mip_level_count(dimensions: &(u32, u32)) -> u32 {
        let max_dim = std::cmp::max(dimensions.0, dimensions.1);
        let levels = (max_dim as f32).log2() as u32;
        levels
    }
}
