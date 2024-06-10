use std::collections::HashMap;
use std::{mem::size_of, sync::Arc};

use crate::runtime::{
    Buffer, BufferProperties, ColorSpace, Image, ImageFormat, ImageProperties, Runtime, Sampler,
};

use crate::shader::{Shader, ShaderLibraryModule};
use crate::utils::math::div_up;

use super::{
    bind_group_manager, BindGroupDescriptor, BindGroupEntry, BindGroupManager, BindingResource,
};

pub struct ImageResizer {
    pipelines: HashMap<ImageFormat, (wgpu::ComputePipeline, BindGroupManager)>,
    uniform_buffer: Buffer,
    texture_sampler: Sampler,
    runtime: Arc<Runtime>,
}
impl ImageResizer {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let uniform_buffer = runtime.create_buffer_of_properties(BufferProperties {
            size: size_of::<f32>(),
            host_readable: false,
        });

        let texture_sampler = runtime.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        ImageResizer {
            runtime,
            pipelines: HashMap::new(),
            uniform_buffer,
            texture_sampler,
        }
    }
}
impl ImageResizer {
    pub fn resize(&mut self, input_img: Arc<Image>, factor: f32) -> Arc<Image> {
        if !self.pipelines.contains_key(&input_img.properties.format) {
            let shader_code = include_str!("./image_resizer.wgsl").replace(
                "IMAGE_FORMAT",
                input_img.properties.format.to_wgsl_format_string(),
            );
            let shader_code = Shader::from_code(shader_code.as_str())
                .with_library(ShaderLibraryModule::ColorSpaces)
                .full_code();

            let (pipeline, bind_group_layout) = self
                .runtime
                .create_compute_pipeline(shader_code.as_str(), Some("ImageResizer"));
            let bind_group_manager = BindGroupManager::new(self.runtime.clone(), bind_group_layout);

            self.pipelines
                .insert(input_img.properties.format, (pipeline, bind_group_manager));
        }

        let (pipeline, bind_group_manager) = self
            .pipelines
            .get_mut(&input_img.properties.format)
            .unwrap();

        bind_group_manager.clear_cache();

        let input_dimensions = input_img.properties.dimensions;
        let output_dimensions = (
            (input_dimensions.0 as f32 * factor) as u32,
            (input_dimensions.1 as f32 * factor) as u32,
        );
        let output_properties = ImageProperties {
            dimensions: output_dimensions,
            ..input_img.properties
        };

        let output_img = Arc::new(self.runtime.create_image_of_properties(output_properties));

        let lod = -factor.log2();

        self.runtime.queue.write_buffer(
            &self.uniform_buffer.buffer,
            0,
            bytemuck::cast_slice(&[lod]),
        );

        let bind_group = bind_group_manager.get_or_create(BindGroupDescriptor {
            entries: vec![
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Texture(&input_img),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.texture_sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureStorage(&output_img, 0),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Buffer(&self.uniform_buffer),
                },
            ],
        });

        let mut encoder = self
            .runtime
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                ..Default::default()
            });
            cpass.set_pipeline(pipeline);

            let num_workgroups_x = div_up(output_dimensions.0, 16);
            let num_workgroups_y = div_up(output_dimensions.1, 16);

            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(num_workgroups_x, num_workgroups_y, 1);
        }
        self.runtime.queue.submit(Some(encoder.finish()));
        output_img
    }
}
