use std::{collections::HashMap, mem::size_of, sync::Arc};

use crate::runtime::Toolbox;

use crate::utils::math::get_cropped_image_dimensions;
use crate::{
    engine::value_store::ValueStore,
    ir::{ApplyFramingOp, Id},
    runtime::{
        BindGroupDescriptor, BindGroupDescriptorKey, BindGroupEntry, BindGroupManager,
        BindingResource, Runtime,
    },
    runtime::{Buffer, BufferProperties, RingBuffer, Sampler},
    runtime::{ColorSpace, ImageProperties},
    shader::{Shader, ShaderLibraryModule},
    utils::math::div_up,
};

pub struct ApplyFramingImpl {
    runtime: Arc<Runtime>,
    pipeline: wgpu::ComputePipeline,
    bind_group_manager: BindGroupManager,
    ring_buffer: RingBuffer,
    texture_sampler: Sampler,
}
impl ApplyFramingImpl {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let shader_code = Shader::from_code(include_str!("shaders/framing.wgsl")).full_code();

        let (pipeline, bind_group_layout) =
            runtime.create_compute_pipeline(shader_code.as_str(), Some("Framing"));

        let bind_group_manager = BindGroupManager::new(runtime.clone(), bind_group_layout);

        let ring_buffer = RingBuffer::new(
            runtime.clone(),
            BufferProperties {
                size: size_of::<f32>(),
                host_readable: false,
            },
        );

        let texture_sampler = runtime.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        ApplyFramingImpl {
            runtime,
            pipeline,
            bind_group_manager,
            ring_buffer,
            texture_sampler,
        }
    }
}
impl ApplyFramingImpl {
    pub fn reset(&mut self) {
        self.bind_group_manager.clear_cache();
    }

    pub fn encode_commands(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        op: &ApplyFramingOp,
        value_store: &mut ValueStore,
        toolbox: &Toolbox,
    ) {
        let input_img = value_store.map.get(&op.arg).unwrap().as_image().clone();

        let input_dimensions = input_img.properties.dimensions;
        let output_aspect_ratio = op.frame.aspect_ratio_float();
        let mut output_dimensions = if output_aspect_ratio >= input_img.aspect_ratio() {
            let output_y = ((1.0 + op.frame.gap) * input_dimensions.1 as f32) as u32;
            let output_x = (output_y as f32 * output_aspect_ratio) as u32;
            (output_x, output_y)
        } else {
            let output_x = ((1.0 + op.frame.gap) * input_dimensions.0 as f32) as u32;
            let output_y = (output_x as f32 / output_aspect_ratio) as u32;
            (output_x, output_y)
        };

        let max_texture_dim = Runtime::get_required_max_texture_dim_1d_2d() as u32;
        let mut factor = 1.0;
        if output_dimensions.0 > max_texture_dim {
            let this_factor = max_texture_dim as f32 / output_dimensions.0 as f32;
            factor *= this_factor;
            output_dimensions.0 = max_texture_dim;
            output_dimensions.1 = (output_dimensions.1 as f32 * this_factor) as u32;
        }
        if output_dimensions.1 > max_texture_dim {
            let this_factor = max_texture_dim as f32 / output_dimensions.1 as f32;
            factor *= this_factor;
            output_dimensions.1 = max_texture_dim;
            output_dimensions.0 = (output_dimensions.0 as f32 * this_factor) as u32;
        }

        let output_properties = ImageProperties {
            dimensions: output_dimensions,
            ..input_img.properties
        };

        let output_img = value_store.ensure_value_at_id_is_image_of_properties(
            self.runtime.as_ref(),
            op.result,
            &output_properties,
        );

        let buffer = self.ring_buffer.get();

        let lod = -factor.log2();

        self.runtime
            .queue
            .write_buffer(&buffer.buffer, 0, bytemuck::cast_slice(&[lod]));

        let bind_group = self.bind_group_manager.get_or_create(BindGroupDescriptor {
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
                    resource: BindingResource::Buffer(buffer),
                },
            ],
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                ..Default::default()
            });
            compute_pass.set_pipeline(&self.pipeline);

            let num_workgroups_x = div_up(output_dimensions.0, 16);
            let num_workgroups_y = div_up(output_dimensions.1, 16);

            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(num_workgroups_x, num_workgroups_y, 1);
        }
    }
}
