use std::{io::Cursor, path::PathBuf, sync::Arc};

use bytemuck::Pod;

use crate::{
    buffer::{Buffer, BufferProperties},
    image::{ColorSpace, Image, ImageFormat, ImageProperties},
    sampler::Sampler,
    utils::{color_space_converter::ColorSpaceConverter, mipmap_generator::MipmapGenerator},
};

use zune_jpeg::JpegDecoder;

use half::prelude::*;

pub struct Runtime {
    pub adapter: Arc<wgpu::Adapter>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,

    toolbox: Option<ToolBox>,
}

struct ToolBox {
    pub mipmap_generator: MipmapGenerator,
    pub color_space_converter: ColorSpaceConverter,
}

impl Runtime {
    pub fn new(
        adapter: Arc<wgpu::Adapter>,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
    ) -> Self {
        let mut runtime = Runtime {
            adapter,
            device,
            queue,
            toolbox: None,
        };
        let toolbox = ToolBox {
            mipmap_generator: MipmapGenerator::new(&runtime),
            color_space_converter: ColorSpaceConverter::new(&runtime),
        };
        runtime.toolbox = Some(toolbox);
        runtime
    }

    pub fn ensure_mipmap(&self, image: &Image) {
        self.toolbox
            .as_ref()
            .unwrap()
            .mipmap_generator
            .generate(self, image);
    }

    pub fn encode_mipmap_generation_command(
        &self,
        image: &Image,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        self.toolbox
            .as_ref()
            .unwrap()
            .mipmap_generator
            .encode_mipmap_generation_command(self, image, encoder);
    }

    pub fn create_compute_pipeline(
        &self,
        wgsl_code: &str,
    ) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(wgsl_code.into()),
            });

        let pipeline = self
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &shader,
                entry_point: "cs_main",
            });

        let bind_group_layout = pipeline.get_bind_group_layout(0);
        (pipeline, bind_group_layout)
    }

    pub fn create_render_pipeline(
        &self,
        wgsl_code: &str,
        target_format: wgpu::TextureFormat,
    ) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(wgsl_code.into()),
            });

        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: None,
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(target_format.into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

        let bind_group_layout = pipeline.get_bind_group_layout(0);
        (pipeline, bind_group_layout)
    }

    pub fn create_image_of_properties(&self, properties: ImageProperties) -> Image {
        let size = wgpu::Extent3d {
            width: properties.dimensions.0,
            height: properties.dimensions.1,
            depth_or_array_layers: 1,
        };
        let mip_level_count = Image::mip_level_count(&properties.dimensions);
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: size,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: properties.format.to_wgpu_texture_format(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut texture_view_single_mip = Vec::new();
        for i in 0..mip_level_count {
            let view = texture.create_view(&wgpu::TextureViewDescriptor {
                base_mip_level: i,
                mip_level_count: Some(1),
                ..Default::default()
            });
            texture_view_single_mip.push(view);
        }

        Image {
            properties,
            texture,
            texture_view,
            texture_view_single_mip,
            uuid: crate::utils::uuid::get_next_uuid(),
        }
    }

    pub fn create_image_from_path(&self, path: &PathBuf) -> Result<Image, String> {
        let Ok(image_bytes) = std::fs::read(&path) else {
            return Err("could not find file at path ".to_string() + path.to_str().unwrap_or(""));
        };
        let Some(ext) = path.extension() else {
            return Err("missing file extension".to_owned());
        };
        self.create_image_from_bytes_and_extension(image_bytes.as_slice(), ext.to_str().unwrap())
    }

    pub fn create_image_from_bytes_and_extension(
        &self,
        image_bytes: &[u8],
        extension: &str,
    ) -> Result<Image, String> {
        let extension = extension.to_lowercase();
        if extension == "jpg" || extension == "jpeg" {
            return self.create_image_from_bytes_jpg(image_bytes);
        }
        Err("unsupported image format: ".to_owned() + extension.as_str())
    }

    pub fn create_image_from_bytes_jpg(&self, image_bytes: &[u8]) -> Result<Image, String> {
        let mut decoder = JpegDecoder::new(image_bytes);
        let pixels_rgb8 = decoder.decode().unwrap();
        let dimensions = decoder.dimensions().unwrap();
        let dimensions = (dimensions.0 as u32, dimensions.1 as u32);

        let properties = ImageProperties {
            dimensions,
            format: ImageFormat::Rgba16Float,
            color_space: ColorSpace::sRGB,
        };
        let result = self.create_image_of_properties(properties);

        let mut pixels_rgba8 = Vec::with_capacity(pixels_rgb8.len() * 4 / 3);

        for i in 0..pixels_rgb8.len() / 3 {
            pixels_rgba8.push(pixels_rgb8[i * 3]);
            pixels_rgba8.push(pixels_rgb8[i * 3 + 1]);
            pixels_rgba8.push(pixels_rgb8[i * 3 + 2]);
            pixels_rgba8.push(255);
        }

        let mut pixels_rgba16 = Vec::with_capacity(pixels_rgba8.len() * 2);
        for i in 0..pixels_rgba8.len() {
            let f = pixels_rgba8[i] as f32 / 255.0;
            let h = f16::from_f32(f);
            let h_bytes = h.to_be_bytes();
            pixels_rgba16.push(h_bytes[1]);
            pixels_rgba16.push(h_bytes[0]);
        }

        let bytes_per_row = dimensions.0 * result.properties.format.bytes_per_pixel();

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
            pixels_rgba16.as_slice(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        self.ensure_mipmap(&result);
        Ok(result)
    }

    pub fn copy_image(&self, src: &Image, dest: &Image) {
        assert!(
            src.properties.dimensions == dest.properties.dimensions,
            "expecting equal dimensions"
        );
        assert!(
            src.properties.format == dest.properties.format,
            "expecting equal dimensions"
        );
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let zero_origin = wgpu::Origin3d { x: 0, y: 0, z: 0 };
        let src_copy = wgpu::ImageCopyTexture {
            texture: &src.texture,
            mip_level: 0,
            origin: zero_origin,
            aspect: wgpu::TextureAspect::All,
        };
        let dest_copy = wgpu::ImageCopyTexture {
            texture: &dest.texture,
            mip_level: 0,
            origin: zero_origin,
            aspect: wgpu::TextureAspect::All,
        };
        let size = wgpu::Extent3d {
            width: src.properties.dimensions.0,
            height: src.properties.dimensions.1,
            depth_or_array_layers: 1,
        };
        encoder.copy_texture_to_texture(src_copy, dest_copy, size);
        self.queue.submit(Some(encoder.finish()));
    }

    pub fn convert_color_space(
        &self,
        input_img: Arc<Image>,
        dest_color_space: ColorSpace,
    ) -> Arc<Image> {
        self.toolbox
            .as_ref()
            .unwrap()
            .color_space_converter
            .convert(self, input_img, dest_color_space)
    }

    pub fn create_sampler(&self, desc: &wgpu::SamplerDescriptor) -> Sampler {
        let sampler = self.device.create_sampler(desc);
        Sampler {
            sampler,
            uuid: crate::utils::uuid::get_next_uuid(),
        }
    }

    // Buffer Stuff

    pub fn create_buffer_of_properties(&self, properties: BufferProperties) -> Buffer {
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: properties.size as u64,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::STORAGE,
        });
        let uuid = crate::utils::uuid::get_next_uuid();
        let mut buffer_host_readable = None;
        if properties.host_readable {
            buffer_host_readable = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: properties.size as u64,
                mapped_at_creation: false,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            }));
        }
        Buffer {
            properties,
            uuid,
            buffer,
            buffer_host_readable,
        }
    }

    pub fn read_buffer<T: Pod>(&self, buffer: &Buffer) -> Vec<T> {
        assert!(
            buffer.properties.host_readable,
            "read_buffer can only be used for host readable buffers"
        );
        assert!(
            buffer.buffer_host_readable.is_some(),
            "missing host readable buffer"
        );
        let buffer_host_readable = buffer.buffer_host_readable.as_ref().unwrap();
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(
            &buffer.buffer,
            0,
            &buffer_host_readable,
            0,
            buffer.properties.size as u64,
        );
        self.queue.submit(Some(encoder.finish()));
        let buffer_slice = buffer_host_readable.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, move |_| {});
        // hacky
        while !self.device.poll(wgpu::Maintain::Wait) {}
        let mapped_range = buffer_slice.get_mapped_range();
        // Since contents are got in bytes, this converts these bytes back to u32
        let result = bytemuck::cast_slice(&mapped_range).to_vec();
        drop(mapped_range);
        buffer_host_readable.unmap();
        result
    }
}
