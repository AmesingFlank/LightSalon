use std::{collections::HashMap, mem::size_of, sync::Arc};

use crate::{
    buffer::{Buffer, BufferProperties, RingBuffer},
    engine::{value_store::ValueStore, toolbox::Toolbox},
    image::ColorSpace,
    ir::{ApplyCurveOp, Id},
    runtime::{
        BindGroupDescriptor, BindGroupDescriptorKey, BindGroupEntry, BindGroupManager,
        BindingResource, Runtime,
    },
    shader::{Shader, ShaderLibraryModule},
    utils::{math::div_up, spline::EvaluatedSpline},
};

pub struct ApplyCurveImpl {
    runtime: Arc<Runtime>,
    pipeline: wgpu::ComputePipeline,
    bind_group_manager: BindGroupManager,
    ring_buffer_curve: RingBuffer,
    ring_buffer_params: RingBuffer,
}

const NUM_STEPS: usize = 255;

impl ApplyCurveImpl {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let shader_code = Shader::from_code(include_str!("shaders/curve.wgsl"))
            .with_library(ShaderLibraryModule::ColorSpaces)
            .full_code();

        let (pipeline, bind_group_layout) = runtime.create_compute_pipeline(shader_code.as_str());

        let bind_group_manager = BindGroupManager::new(runtime.clone(), bind_group_layout);

        let ring_buffer_curve = RingBuffer::new(
            runtime.clone(),
            BufferProperties {
                size: size_of::<f32>() * (NUM_STEPS + 1) + size_of::<f32>(),
                host_readable: false,
            },
        );

        let ring_buffer_params = RingBuffer::new(
            runtime.clone(),
            BufferProperties {
                size: size_of::<u32>() * 3,
                host_readable: false,
            },
        );

        ApplyCurveImpl {
            runtime,
            pipeline,
            bind_group_manager,
            ring_buffer_curve,
            ring_buffer_params,
        }
    }
}
impl ApplyCurveImpl {
    pub fn reset(&mut self) {
        self.ring_buffer_curve.mark_all_available();
        self.ring_buffer_params.mark_all_available();
    }

    pub fn encode_commands(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        op: &ApplyCurveOp,
        value_store: &mut ValueStore,
        toolbox: &mut Toolbox,
    ) {
        let input_img = value_store.map.get(&op.arg).unwrap().as_image().clone();
        let output_img = value_store.ensure_value_at_id_is_image_of_properties(
            self.runtime.as_ref(),
            op.result,
            &input_img.properties,
        );

        let buffer_curve = self.ring_buffer_curve.get();

        let evaluated =
            EvaluatedSpline::from_control_points(&op.control_points, 1.0, NUM_STEPS as u32);
        evaluated.write_to_buffer(&self.runtime, buffer_curve);

        let buffer_params = self.ring_buffer_params.get();
        self.runtime.queue.write_buffer(
            &buffer_params.buffer,
            0,
            bytemuck::cast_slice(&[op.apply_r as u32, op.apply_g as u32, op.apply_b as u32]),
        );

        let bind_group = self.bind_group_manager.get_or_create(BindGroupDescriptor {
            entries: vec![
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Texture(&input_img),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureStorage(&output_img, 0),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Buffer(buffer_curve),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Buffer(buffer_params),
                },
            ],
        });

        {
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            compute_pass.set_pipeline(&self.pipeline);

            let num_workgroups_x = div_up(input_img.properties.dimensions.0, 16);
            let num_workgroups_y = div_up(input_img.properties.dimensions.1, 16);

            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(num_workgroups_x, num_workgroups_y, 1);
        }
    }
}
