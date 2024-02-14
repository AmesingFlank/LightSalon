use crate::runtime::BufferProperties;
use std::sync::Arc;

use super::{Buffer, Runtime};

pub struct Image {
    pub properties: ImageProperties,
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub texture_view_single_mip: Vec<wgpu::TextureView>,
    pub uuid: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    // matches color_spaces.wgsl
    LinearRGB = 0,
    sRGB = 1,
    HSL = 2,
    LCh = 3,
    HSLuv = 4,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageFormat {
    Rgba16Float,
    Rgba8Unorm,
}

impl ImageFormat {
    pub fn to_wgpu_texture_format(&self) -> wgpu::TextureFormat {
        match *self {
            ImageFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
            ImageFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
        }
    }
    pub fn to_wgsl_format_string(&self) -> &str {
        match *self {
            ImageFormat::Rgba16Float => "rgba16float",
            ImageFormat::Rgba8Unorm => "rgba8unorm",
        }
    }
    pub fn bytes_per_channel(&self) -> u32 {
        match *self {
            ImageFormat::Rgba16Float => 2,
            ImageFormat::Rgba8Unorm => 1,
        }
    }
    pub fn bytes_per_pixel(&self) -> u32 {
        match *self {
            ImageFormat::Rgba16Float => 8,
            ImageFormat::Rgba8Unorm => 4,
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

    pub fn get_lowest_rendered_mip(
        image_dimensions: (u32, u32),
        rendered_dimensions: (u32, u32),
    ) -> u32 {
        let x_ratio = image_dimensions.0 as f32 / rendered_dimensions.0 as f32;
        let x_lod = x_ratio.log2().floor() as u32;
        let y_ratio = image_dimensions.1 as f32 / rendered_dimensions.1 as f32;
        let y_lod = y_ratio.log2().floor() as u32;
        std::cmp::min(x_lod, y_lod)
    }
}

pub struct ImageReader {
    runtime: Arc<Runtime>,
    image: Arc<Image>,
    buffer: Arc<Buffer>,
    map_ready_receiver: flume::Receiver<()>,
    result_dynamic_image: Option<image::DynamicImage>,
    pending_read: bool,
}

impl ImageReader {
    pub fn new(runtime: Arc<Runtime>, image: Arc<Image>) -> Self {
        assert!(
            image.properties.format == ImageFormat::Rgba8Unorm,
            "only reading Rgba8Unorm is supported"
        );
        let image_data_size = image.properties.dimensions.0
            * image.properties.dimensions.1
            * image.properties.format.bytes_per_pixel();
        let buffer = runtime.create_buffer_of_properties(BufferProperties {
            size: image_data_size as usize,
            host_readable: true,
        });
        let map_ready_receiver = runtime.map_host_readable_buffer(&buffer);
        Self {
            runtime,
            image,
            buffer: Arc::new(buffer),
            map_ready_receiver,
            result_dynamic_image: None,
            pending_read: true,
        }
    }

    pub fn take_dynamic_image(&mut self) -> Option<image::DynamicImage> {
        self.result_dynamic_image.take()
    }

    pub fn poll_dynamic_image(&mut self) -> Option<&image::DynamicImage> {
        if self.pending_read {
            if let Ok(_) = self.map_ready_receiver.try_recv() {
                self.read_dynamic_image_from_mapped_buffer();
            }
        }
        self.result_dynamic_image.as_ref()
    }

    pub async fn await_dynamic_image(&mut self) -> Option<&image::DynamicImage> {
        if self.pending_read {
            if let Ok(_) = self.map_ready_receiver.recv_async().await {
                self.read_dynamic_image_from_mapped_buffer();
            } else {
                panic!("recv_async().await failed")
            }
        }
        self.result_dynamic_image.as_ref()
    }

    fn read_dynamic_image_from_mapped_buffer(&mut self) {
        let data: Vec<u8> = self.runtime.read_mapped_buffer(&self.buffer);
        let dynamic_image = self.runtime.create_dynamic_image_from_bytes_jpg_png(&data);
        let dynamic_image = dynamic_image.expect("Couldn't read image");
        self.result_dynamic_image = Some(dynamic_image);
        self.pending_read = false;
    }

    pub fn pending_read(&self) -> bool {
        self.result_dynamic_image.is_none()
    }

    pub fn image(&self) -> &Arc<Image> {
        &self.image
    }
}
