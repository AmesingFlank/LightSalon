use std::{mem::size_of, sync::Arc};

use crate::{
    engine::value_store::ValueStore,
    image::ColorSpace,
    ir::{AdjustExposureOp, ComputeHistogramOp},
    runtime::Runtime,
    shader::{Shader, ShaderLibraryModule}, buffer::BufferProperties,
};

pub struct ComputeHistogramImpl {
    runtime: Arc<Runtime>,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout, 
}
impl ComputeHistogramImpl {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let shader_code = Shader::from_code(include_str!("./histogram.wgsl"))
            .with_library(ShaderLibraryModule::ColorSpaces)
            .full_code();

        let (pipeline, bind_group_layout) = runtime.create_compute_pipeline(shader_code.as_str()); 
        ComputeHistogramImpl {
            runtime,
            pipeline,
            bind_group_layout,
        }
    }
}
impl ComputeHistogramImpl {
    pub fn apply(&mut self, op: &ComputeHistogramOp, value_store: &mut ValueStore) {
        let input_img = value_store.map.get(&op.arg).unwrap().as_image().clone();
        assert!(
            input_img.properties.color_space == ColorSpace::Linear,
            "expecting linear color space"
        );

        let buffer_props = BufferProperties {
            size: 4 * 256 * size_of::<f32>()
        };

        let output_buffer = value_store.ensure_value_at_id_is_buffer_of_properties(
            self.runtime.as_ref(),
            op.result,
            &buffer_props,
        );

        self.runtime.queue.write_buffer(
            &output_buffer.buffer,
            0,
            bytemuck::cast_slice(&[0.0; 4 * 256]),
        );

        let bind_group = self
            .runtime
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&input_img.texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: output_buffer.buffer.as_entire_binding(),
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
