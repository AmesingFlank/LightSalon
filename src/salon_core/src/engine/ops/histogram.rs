use std::{mem::size_of, sync::Arc};

use crate::runtime::Toolbox;

use crate::{
    engine::{common::ImageHistogram, value_store::ValueStore},
    ir::{ComputeHistogramOp},
    runtime::{
        BindGroupDescriptor, BindGroupEntry, BindGroupManager,
        BindingResource, Runtime,
    },
    runtime::{BufferProperties, RingBuffer},
    shader::{Shader, ShaderLibraryModule},
    utils::math::div_up,
};

pub struct ComputeHistogramImpl {
    runtime: Arc<Runtime>,

    ring_buffer: RingBuffer,

    pipeline_clear: wgpu::ComputePipeline,
    bind_group_manager_clear: BindGroupManager,

    pipeline_compute: wgpu::ComputePipeline,
    bind_group_manager_compute: BindGroupManager,
}
impl ComputeHistogramImpl {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let shader_clear =
            Shader::from_code(include_str!("shaders/histogram_clear.wgsl")).full_code();
        let (pipeline_clear, bind_group_layout_clear) =
            runtime.create_compute_pipeline(shader_clear.as_str(), Some("HistogramClear"));
        let bind_group_manager_clear =
            BindGroupManager::new(runtime.clone(), bind_group_layout_clear);

        let shader_compute = Shader::from_code(include_str!("shaders/histogram_compute.wgsl"))
            .with_library(ShaderLibraryModule::ColorSpaces)
            .with_library(ShaderLibraryModule::Random)
            .full_code();
        let (pipeline_compute, bind_group_layout_compute) =
            runtime.create_compute_pipeline(shader_compute.as_str(), Some("HistogramCompute"));
        let bind_group_manager_compute =
            BindGroupManager::new(runtime.clone(), bind_group_layout_compute);

        let ring_buffer = RingBuffer::new(
            runtime.clone(),
            BufferProperties {
                size: size_of::<f32>(),
                host_readable: false,
            },
        );

        ComputeHistogramImpl {
            runtime,
            ring_buffer,
            pipeline_clear,
            bind_group_manager_clear,
            pipeline_compute,
            bind_group_manager_compute,
        }
    }
}
impl ComputeHistogramImpl {
    pub fn reset(&mut self) {
        self.ring_buffer.mark_all_available();
        self.bind_group_manager_clear.clear_cache();
        self.bind_group_manager_compute.clear_cache();
    }

    pub fn encode_commands(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        op: &ComputeHistogramOp,
        value_store: &mut ValueStore,
        _toolbox: &Toolbox,
    ) {
        let input_img = value_store.map.get(&op.arg).unwrap().as_image().clone();

        let buffer_props = BufferProperties {
            size: (4 * ImageHistogram::max_bins() + 1) * size_of::<u32>(),
            host_readable: true,
        };

        let output_buffer = value_store.ensure_value_at_id_is_buffer_of_properties(
            self.runtime.as_ref(),
            op.result,
            &buffer_props,
        );

        let uniform_buffer = self.ring_buffer.get();
        let num_bins = ImageHistogram::num_bins_for(input_img.properties.dimensions);
        self.runtime.queue.write_buffer(
            &uniform_buffer.buffer,
            0,
            bytemuck::cast_slice(&[num_bins as u32]),
        );

        let bind_group_clear = self
            .bind_group_manager_clear
            .get_or_create(BindGroupDescriptor {
                entries: vec![BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(&output_buffer),
                }],
            });

        let bind_group_compute =
            self.bind_group_manager_compute
                .get_or_create(BindGroupDescriptor {
                    entries: vec![
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::Texture(&input_img),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::Buffer(uniform_buffer),
                        },
                        BindGroupEntry {
                            binding: 2,
                            resource: BindingResource::Buffer(&output_buffer),
                        },
                    ],
                });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                ..Default::default()
            });

            compute_pass.set_pipeline(&self.pipeline_clear);
            compute_pass.set_bind_group(0, &bind_group_clear, &[]);
            compute_pass.dispatch_workgroups(ImageHistogram::max_bins() as u32, 1, 1);

            compute_pass.set_pipeline(&self.pipeline_compute);
            compute_pass.set_bind_group(0, &bind_group_compute, &[]);

            let num_workgroups_x = div_up(input_img.properties.dimensions.0, 16);
            let num_workgroups_y = div_up(input_img.properties.dimensions.1, 16);

            compute_pass.dispatch_workgroups(num_workgroups_x, num_workgroups_y, 1);
        }
    }
}
