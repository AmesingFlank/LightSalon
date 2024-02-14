use std::{io::Cursor, path::PathBuf, sync::Arc};

use bytemuck::Pod;
use image::{buffer, imageops, DynamicImage, GenericImageView, ImageBuffer, Rgb};

use crate::runtime::{
    buffer::{Buffer, BufferProperties},
    image::{ColorSpace, Image, ImageFormat, ImageProperties},
    sampler::Sampler,
};

pub struct Runtime {
    pub adapter: Arc<wgpu::Adapter>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
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
        };
        runtime
    }

    pub fn create_compute_pipeline(
        &self,
        wgsl_code: &str,
        label: Option<&str>,
    ) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label,
                source: wgpu::ShaderSource::Wgsl(wgsl_code.into()),
            });

        let pipeline = self
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label,
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
        label: Option<&str>,
    ) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label,
                source: wgpu::ShaderSource::Wgsl(wgsl_code.into()),
            });

        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label,
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
        if extension == "jpg" || extension == "jpeg" || extension == "png" {
            return self.create_image_from_bytes_jpg_png(image_bytes);
        }
        Err("unsupported image format: ".to_owned() + extension.as_str())
    }

    pub fn create_image_from_bytes_jpg_png(&self, image_bytes: &[u8]) -> Result<Image, String> {
        let img = self.create_dynamic_image_from_bytes_jpg_png(image_bytes)?;
        Ok(self.create_image_from_dynamic_image(img))
    }

    pub fn create_dynamic_image_from_bytes_jpg_png(
        &self,
        image_bytes: &[u8],
    ) -> Result<DynamicImage, String> {
        let Ok(mut img) = image::load_from_memory(image_bytes) else {
            return Err("image::load_from_memory failed".to_owned());
        };

        // use exif to fix image orientation
        // https://github.com/image-rs/image/issues/1958
        let exif_reader = exif::Reader::new();
        let mut cursor = Cursor::new(image_bytes);
        let exif = exif_reader.read_from_container(&mut cursor);

        let orientation: u32 = match exif {
            Ok(exif) => match exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY) {
                Some(orientation) => match orientation.value.get_uint(0) {
                    Some(v @ 1..=8) => v,
                    _ => 1,
                },
                None => 1,
            },
            Err(_) => 1,
        };

        if orientation == 2 {
            img = DynamicImage::ImageRgba8(imageops::flip_horizontal(&img));
        } else if orientation == 3 {
            img = DynamicImage::ImageRgba8(imageops::rotate180(&img));
        } else if orientation == 4 {
            img = DynamicImage::ImageRgba8(imageops::flip_horizontal(&img));
        } else if orientation == 5 {
            img = DynamicImage::ImageRgba8(imageops::rotate90(&img));
            img = DynamicImage::ImageRgba8(imageops::flip_horizontal(&img));
        } else if orientation == 6 {
            img = DynamicImage::ImageRgba8(imageops::rotate90(&img));
        } else if orientation == 7 {
            img = DynamicImage::ImageRgba8(imageops::rotate270(&img));
            img = DynamicImage::ImageRgba8(imageops::flip_horizontal(&img));
        } else if orientation == 8 {
            img = DynamicImage::ImageRgba8(imageops::rotate270(&img));
        }
        Ok(img)
    }

    pub fn create_image_from_dynamic_image(&self, dynamic_image: image::DynamicImage) -> Image {
        let dimensions = dynamic_image.dimensions();
        let properties = ImageProperties {
            dimensions,
            format: ImageFormat::Rgba8Unorm,
            color_space: ColorSpace::sRGB,
        };
        let result = self.create_image_of_properties(properties);

        let image_buffer_rgba8 = dynamic_image.to_rgba8();
        let image_bytes = image_buffer_rgba8.as_raw();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let bytes_per_row = dimensions.0 * result.properties.format.bytes_per_pixel();

        self.queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &result.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            image_bytes.as_slice(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        result
    }

    pub fn copy_image(&self, src: &Image, dest: &Image) {
        assert!(
            src.properties.dimensions == dest.properties.dimensions,
            "expecting equal dimensions"
        );
        assert!(
            src.properties.format == dest.properties.format,
            "expecting equal format"
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

    pub fn copy_image_to_host_readable_buffer(&self, image: &Image, buffer: &Buffer) {
        let bytes_per_row =
            image.properties.dimensions.0 * image.properties.format.bytes_per_pixel();
        let image_data_size = bytes_per_row * image.properties.dimensions.1;
        assert!(
            image_data_size as usize <= buffer.properties.size,
            "buffer not big enough to hold the image"
        );
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
        let zero_origin = wgpu::Origin3d { x: 0, y: 0, z: 0 };
        let src_copy = wgpu::ImageCopyTexture {
            texture: &image.texture,
            mip_level: 0,
            origin: zero_origin,
            aspect: wgpu::TextureAspect::All,
        };
        let dest_copy = wgpu::ImageCopyBuffer {
            buffer: buffer_host_readable,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(image.properties.dimensions.1),
            },
        };
        let size = wgpu::Extent3d {
            width: image.properties.dimensions.0,
            height: image.properties.dimensions.1,
            depth_or_array_layers: 1,
        };
        encoder.copy_texture_to_buffer(src_copy, dest_copy, size);
        self.queue.submit(Some(encoder.finish()));
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

    pub fn map_host_readable_buffer(&self, buffer: &Buffer) -> flume::Receiver<()> {
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
        let (sender, receiver) = flume::bounded(1);
        buffer_slice.map_async(wgpu::MapMode::Read, move |_r| {
            // if the receiver has been dropped this might fail, but that's OK?
            let _ = sender.send(());
        });
        self.device.poll(wgpu::Maintain::wait()).panic_on_timeout();
        receiver
    }

    pub fn read_mapped_buffer<T: Pod + std::marker::Send>(&self, buffer: &Buffer) -> Vec<T> {
        assert!(
            buffer.properties.host_readable,
            "read_buffer can only be used for host readable buffers"
        );
        assert!(
            buffer.buffer_host_readable.is_some(),
            "missing host readable buffer"
        );
        let buffer_host_readable = buffer.buffer_host_readable.as_ref().unwrap();
        let buffer_slice = buffer_host_readable.slice(..);

        let mapped_range = buffer_slice.get_mapped_range();
        let result: Vec<T> = bytemuck::cast_slice(&mapped_range).to_vec();
        drop(mapped_range);
        buffer_host_readable.unmap();

        result
    }
}
