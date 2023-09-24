use std::{collections::HashMap, mem::size_of, sync::Arc};

use crate::{engine::Op, image::Image, runtime::Runtime};

pub struct ExposureOp {
    runtime: Arc<Runtime>,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
}
impl ExposureOp {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let (pipeline, bind_group_layout) =
            runtime.create_compute_pipeline(include_str!("./exposure.wgsl"));

        let uniform_buffer = runtime.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size_of::<f32>() as u64,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        ExposureOp {
            runtime,
            pipeline,
            bind_group_layout,
            uniform_buffer,
        }
    }
}
impl Op for ExposureOp {
    fn apply(
        &mut self,
        inputs: Vec<Arc<Image>>,
        outputs: Vec<Arc<Image>>,
        params: serde_json::Value,
    ) {
        assert!(
            inputs.len() == outputs.len(),
            "expecting inputs and outputs to have equal size"
        );
        let value = params.as_f64().unwrap() as f32;
        self.runtime.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[value]),
        );

        let mut bind_groups = Vec::new();
        for i in 0..inputs.len() {
            let bind_group = self
                .runtime
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&inputs[i].texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&outputs[i].texture_view_base_mip),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: self.uniform_buffer.as_entire_binding(),
                        },
                    ],
                });
            bind_groups.push(bind_group);
        }

        let mut encoder = self
            .runtime
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.pipeline);
            for i in 0..inputs.len() {
                cpass.set_bind_group(0, &bind_groups[i], &[]);
                cpass.dispatch_workgroups(inputs[i].dimensions.0, inputs[i].dimensions.1, 1);
            }
        }
        for i in 0..inputs.len() {
            self.runtime.encode_mipmap_generation_command(outputs[i].as_ref(), &mut encoder);
        }
        self.runtime.queue.submit(Some(encoder.finish()));
    }
}
