use image::GenericImageView;

use crate::runtime;
pub struct Image {
    pub dimensions: (u32, u32),
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
}

impl Image {
    pub fn create_from_bytes(runtime: &runtime::Runtime) -> Self {
        let bytes = include_bytes!("../../../assets/images/car.jpg");
        let image = image::load_from_memory(bytes).unwrap();
        let rgba = image.to_rgba8();

        let dimensions = image.dimensions();
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = runtime.device.create_texture(&wgpu::TextureDescriptor {
            size: size,
            mip_level_count: 1, 
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            label: None,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        runtime.queue.write_texture(
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
            texture_view
        }
    }
}
