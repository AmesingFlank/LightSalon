use std::{collections::HashMap, sync::Arc};

use crate::{image::Image, runtime::Runtime};

pub struct ExposureOp {
    runtime: Arc<Runtime>,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}
impl ExposureOp {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let (pipeline, bind_group_layout) =
            runtime.create_compute_pipeline(include_str!("./exposure.wgsl"));

        ExposureOp {
            runtime,
            pipeline,
            bind_group_layout,
        }
    }

    pub fn apply<'img>(
        &'img mut self,
        input: &'img Image,
        output: &'img Image,
        from_value: f32,
        to_value: f32,
    ) {
        let bind_group = self
            .runtime
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&input.texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&output.texture_view),
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
            cpass.dispatch_workgroups(input.dimensions.0, input.dimensions.1, 1);
        }
        self.runtime.queue.submit(Some(encoder.finish()));
    }
}
