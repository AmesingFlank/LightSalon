use std::{sync::Arc, path::PathBuf};

use image::GenericImageView;

use crate::image::Image;

pub struct Runtime {
    pub adapter: Arc<wgpu::Adapter>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
}

impl Runtime {
    pub fn create_image_from_path(&self, path: &PathBuf) -> Result<Image, String> {
        let img = image::open(path.clone());
        match img {
            Ok(i) => Ok(self.create_image_from_dynamic_image(i)),
            Err(_) => {
                Err("could not open image at path ".to_string() + path.to_str().unwrap_or(""))
            }
        }
    }
    
    pub fn create_image_from_dynamic_image(&self, image: image::DynamicImage) -> Image {
        let rgba = image.to_rgba8();

        let dimensions = image.dimensions();
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

        self.queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &texture,
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

        Image {
            dimensions,
            texture,
            texture_view,
            uuid: crate::uuid::get_next_uuid(),
        }
    }
}
