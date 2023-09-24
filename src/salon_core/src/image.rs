use std::path::PathBuf;
use image::GenericImageView;

use crate::runtime;
pub struct Image {
    pub dimensions: (u32, u32),
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub uuid: u32,
}

impl Image {
    pub fn aspect_ratio(&self) -> f32 {
        self.dimensions.1 as f32 / self.dimensions.0 as f32
    }
    pub fn mip_level_count(dimensions: &(u32, u32)) -> u32 {
        let max_dim = std::cmp::max(dimensions.0, dimensions.1);
        let levels = (max_dim as f32).log2() as u32;
        levels
    }
}

