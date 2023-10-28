use std::{collections::HashMap, mem::size_of, sync::Arc};

use crate::{
    buffer::{Buffer, BufferProperties, RingBuffer},
    engine::value_store::ValueStore,
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
    ring_buffer: RingBuffer,
}

const NUM_STEPS: usize = 255;

impl ApplyCurveImpl {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let shader_code = Shader::from_code(include_str!("shaders/curve.wgsl"))
            .with_library(ShaderLibraryModule::ColorSpaces)
            .full_code();

        let (pipeline, bind_group_layout) = runtime.create_compute_pipeline(shader_code.as_str());

        let bind_group_manager = BindGroupManager::new(runtime.clone(), bind_group_layout);

        let ring_buffer = RingBuffer::new(
            runtime.clone(),
            BufferProperties {
                size: size_of::<f32>() * (NUM_STEPS + 1) + size_of::<f32>(),
                host_readable: false,
            },
        );

        ApplyCurveImpl {
            runtime,
            pipeline,
            bind_group_manager,
            ring_buffer,
        }
    }
}
impl ApplyCurveImpl {
    pub fn reset(&mut self) {
        self.ring_buffer.mark_all_available();
    }

    pub fn encode_commands(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        op: &ApplyCurveOp,
        value_store: &mut ValueStore,
    ) {
        let input_img = value_store.map.get(&op.arg).unwrap().as_image().clone();
        let output_img = value_store.ensure_value_at_id_is_image_of_properties(
            self.runtime.as_ref(),
            op.result,
            &input_img.properties,
        );

        let buffer = self.ring_buffer.get();

        let evaluated =
            EvaluatedSpline::from_control_points(&op.control_points, 1.0, NUM_STEPS as u32);
        evaluated.write_to_buffer(&self.runtime, buffer);

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
                    resource: BindingResource::Buffer(buffer),
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
