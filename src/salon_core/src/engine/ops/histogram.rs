use std::{mem::size_of, sync::Arc};

use crate::{
    buffer::BufferProperties,
    engine::value_store::ValueStore,
    image::ColorSpace,
    ir::ComputeHistogramOp,
    runtime::{BindGroupDescriptor, BindGroupEntry, BindGroupManager, BindingResource, Runtime},
    shader::{Shader, ShaderLibraryModule},
};

pub struct ComputeHistogramImpl {
    runtime: Arc<Runtime>,
    pipeline: wgpu::ComputePipeline,
    bind_group_manager: BindGroupManager,
}
impl ComputeHistogramImpl {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let shader_code = Shader::from_code(include_str!("./histogram.wgsl"))
            .with_library(ShaderLibraryModule::ColorSpaces)
            .full_code();

        let (pipeline, bind_group_layout) = runtime.create_compute_pipeline(shader_code.as_str());
        let bind_group_manager = BindGroupManager::new(runtime.clone(), bind_group_layout);

        ComputeHistogramImpl {
            runtime,
            pipeline,
            bind_group_manager,
        }
    }
}
impl ComputeHistogramImpl {
    pub fn prepare(&mut self) {
        
    }
    pub fn apply(&mut self, op: &ComputeHistogramOp, value_store: &mut ValueStore) {
        let input_img = value_store.map.get(&op.arg).unwrap().as_image().clone();
        assert!(
            input_img.properties.color_space == ColorSpace::Linear,
            "expecting linear color space"
        );

        let buffer_props = BufferProperties {
            size: 4 * 256 * size_of::<u32>(),
        };

        let output_buffer = value_store.ensure_value_at_id_is_buffer_of_properties(
            self.runtime.as_ref(),
            op.result,
            &buffer_props,
        );

        self.runtime.queue.write_buffer(
            &output_buffer.buffer,
            0,
            bytemuck::cast_slice(&[0u32; 4 * 256]),
        );

        let bind_group = self.bind_group_manager.get_or_create(BindGroupDescriptor {
            entries: vec![
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Texture(&input_img),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Buffer(&output_buffer),
                },
            ],
        });

        let mut encoder = self
            .runtime
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.pipeline);

            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(
                input_img.properties.dimensions.0,
                input_img.properties.dimensions.1,
                1,
            );
        }
        self.runtime.queue.submit(Some(encoder.finish()));
    }
}
