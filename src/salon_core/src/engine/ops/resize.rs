use std::{mem::size_of, sync::Arc};

use crate::runtime::Toolbox;

use crate::{
    engine::value_store::ValueStore,
    ir::{ResizeOp},
    runtime::{
        BindGroupDescriptor, BindGroupEntry, BindGroupManager,
        BindingResource, Runtime,
    },
    runtime::{BufferProperties, RingBuffer, Sampler},
    runtime::{ImageProperties},
    shader::{Shader},
    utils::math::div_up,
};

pub struct ResizeImpl {
    runtime: Arc<Runtime>,
    pipeline: wgpu::ComputePipeline,
    bind_group_manager: BindGroupManager,
    ring_buffer: RingBuffer,
    texture_sampler: Sampler,
}
impl ResizeImpl {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let shader_code = Shader::from_code(include_str!("shaders/resize.wgsl")).full_code();

        let (pipeline, bind_group_layout) =
            runtime.create_compute_pipeline(shader_code.as_str(), Some("Resize"));

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

        ResizeImpl {
            runtime,
            pipeline,
            bind_group_manager,
            ring_buffer,
            texture_sampler,
        }
    }
}
impl ResizeImpl {
    pub fn reset(&mut self) {
        self.ring_buffer.mark_all_available();
        self.bind_group_manager.clear_cache();
    }

    pub fn encode_commands(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        op: &ResizeOp,
        value_store: &mut ValueStore,
        toolbox: &Toolbox,
    ) {
        let input_img = value_store.map.get(&op.arg).unwrap().as_image().clone();

        toolbox.encode_mipmap_generation_command(&input_img, encoder);

        let input_dimensions = input_img.properties.dimensions;
        let output_dimensions = (
            (input_dimensions.0 as f32 * op.factor) as u32,
            (input_dimensions.1 as f32 * op.factor) as u32,
        );
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

        let lod = -op.factor.log2();

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
