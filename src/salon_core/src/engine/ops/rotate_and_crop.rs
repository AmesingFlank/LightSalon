use std::{collections::HashMap, mem::size_of, sync::Arc};

use crate::runtime::Toolbox;

use crate::utils::math::get_cropped_image_dimensions;
use crate::{
    engine::value_store::ValueStore,
    ir::{Id, RotateAndCropOp},
    runtime::{
        BindGroupDescriptor, BindGroupDescriptorKey, BindGroupEntry, BindGroupManager,
        BindingResource, Runtime,
    },
    runtime::{Buffer, BufferProperties, RingBuffer, Sampler},
    runtime::{ColorSpace, ImageProperties},
    shader::{Shader, ShaderLibraryModule},
    utils::math::div_up,
};

pub struct RotateAndCropImpl {
    runtime: Arc<Runtime>,
    pipeline: wgpu::ComputePipeline,
    bind_group_manager: BindGroupManager,
    ring_buffer: RingBuffer,
    texture_sampler: Sampler,
}
impl RotateAndCropImpl {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let shader_code =
            Shader::from_code(include_str!("shaders/rotate_and_crop.wgsl")).full_code();

        let (pipeline, bind_group_layout) =
            runtime.create_compute_pipeline(shader_code.as_str(), Some("Crop"));

        let bind_group_manager = BindGroupManager::new(runtime.clone(), bind_group_layout);

        let ring_buffer = RingBuffer::new(
            runtime.clone(),
            BufferProperties {
                size: size_of::<f32>() * 3,
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

        RotateAndCropImpl {
            runtime,
            pipeline,
            bind_group_manager,
            ring_buffer,
            texture_sampler,
        }
    }
}
impl RotateAndCropImpl {
    pub fn reset(&mut self) {
        self.ring_buffer.mark_all_available();
    }

    pub fn encode_commands(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        op: &RotateAndCropOp,
        value_store: &mut ValueStore,
        toolbox: &Toolbox,
    ) {
        let input_img = value_store.map.get(&op.arg).unwrap().as_image().clone();

        let input_dimensions = input_img.properties.dimensions;
        let output_dimensions = get_cropped_image_dimensions(input_dimensions, op.crop_rect);
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

        let rotation_radians = op.rotation_degrees.to_radians();

        self.runtime.queue.write_buffer(
            &buffer.buffer,
            0,
            bytemuck::cast_slice(&[
                op.crop_rect.center.x,
                op.crop_rect.center.y,
                rotation_radians,
            ]),
        );

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
