use std::{path::PathBuf, sync::Arc};

use image::GenericImageView;

use crate::image::Image;

pub struct Runtime {
    pub adapter: Arc<wgpu::Adapter>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
}

impl Runtime {
    pub fn create_image_of_size(&self, dimensions: (u32, u32)) -> Image {
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            label: None,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Image {
            dimensions,
            texture,
            texture_view,
            uuid: crate::uuid::get_next_uuid(),
        }
    }

    pub fn create_image_from_path(&self, path: &PathBuf) -> Result<Image, String> {
        let img = image::open(path.clone());
        match img {
            Ok(dynamic_image) => Ok(self.create_image_from_dynamic_image(dynamic_image)),
            Err(_) => {
                Err("could not open image at path ".to_string() + path.to_str().unwrap_or(""))
            }
        }
    }

    pub fn create_image_from_dynamic_image(&self, dynamic_image: image::DynamicImage) -> Image {
        let dimensions = dynamic_image.dimensions();
        let result = self.create_image_of_size(dimensions);
        let rgba = dynamic_image.to_rgba8();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        self.queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &result.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );
        result
    }
}
