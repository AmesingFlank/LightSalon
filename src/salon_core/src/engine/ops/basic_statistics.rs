use std::{collections::HashMap, mem::size_of, sync::Arc};

use crate::runtime::Toolbox;

use crate::{
    engine::{value_store::ValueStore},
    ir::{ComputeBasicStatisticsOp, Id},
    runtime::{
        BindGroupDescriptor, BindGroupDescriptorKey, BindGroupEntry, BindGroupManager,
        BindingResource, Runtime,
    },
    runtime::{BufferProperties, RingBuffer},
    runtime::{ColorSpace, Image},
    shader::{Shader, ShaderLibraryModule},
    utils::math::div_up,
};

pub struct ComputeBasicStatisticsImpl {
    runtime: Arc<Runtime>,

    ring_buffer: RingBuffer,

    pipeline_clear: wgpu::ComputePipeline,
    bind_group_manager_clear: BindGroupManager,

    pipeline_sum: wgpu::ComputePipeline,
    bind_group_manager_sum: BindGroupManager,

    pipeline_divide: wgpu::ComputePipeline,
    bind_group_manager_divide: BindGroupManager,
}
impl ComputeBasicStatisticsImpl {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let shader_clear =
            Shader::from_code(include_str!("shaders/basic_statistics_clear.wgsl")).full_code();
        let (pipeline_clear, bind_group_layout_clear) =
            runtime.create_compute_pipeline(shader_clear.as_str(), Some("BasicStatisticsClear"));
        let bind_group_manager_clear =
            BindGroupManager::new(runtime.clone(), bind_group_layout_clear);

        let shader_sum =
            Shader::from_code(include_str!("shaders/basic_statistics_sum.wgsl")).full_code();
        let (pipeline_sum, bind_group_layout_sum) =
            runtime.create_compute_pipeline(shader_sum.as_str(), Some("BasicStatisticsSum"));
        let bind_group_manager_sum = BindGroupManager::new(runtime.clone(), bind_group_layout_sum);

        let shader_divide =
            Shader::from_code(include_str!("shaders/basic_statistics_divide.wgsl")).full_code();
        let (pipeline_divide, bind_group_layout_divide) =
            runtime.create_compute_pipeline(shader_divide.as_str(), Some("BasicStatisticsDivide"));
        let bind_group_manager_divide =
            BindGroupManager::new(runtime.clone(), bind_group_layout_divide);

        let ring_buffer = RingBuffer::new(
            runtime.clone(),
            BufferProperties {
                size: 4 * size_of::<u32>(),
                host_readable: false,
            },
        );

        ComputeBasicStatisticsImpl {
            runtime,
            ring_buffer,
            pipeline_clear,
            bind_group_manager_clear,
            pipeline_sum,
            bind_group_manager_sum,
            pipeline_divide,
            bind_group_manager_divide,
        }
    }
}
impl ComputeBasicStatisticsImpl {
    pub fn reset(&mut self) {
        self.ring_buffer.mark_all_available();
    }

    pub fn encode_commands(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        op: &ComputeBasicStatisticsOp,
        value_store: &mut ValueStore,
        toolbox: &Toolbox,
    ) {
        let input_img = value_store.map.get(&op.arg).unwrap().as_image().clone();

        let buffer_props = BufferProperties {
            size: 4 * size_of::<f32>(),
            host_readable: true,
        };

        let output_buffer = value_store.ensure_value_at_id_is_buffer_of_properties(
            self.runtime.as_ref(),
            op.result,
            &buffer_props,
        );

        let working_buffer = self.ring_buffer.get();

        let bind_group_clear = self
            .bind_group_manager_clear
            .get_or_create(BindGroupDescriptor {
                entries: vec![BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(working_buffer),
                }],
            });

        let bind_group_sum = self
            .bind_group_manager_sum
            .get_or_create(BindGroupDescriptor {
                entries: vec![
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::Texture(&input_img),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Buffer(&working_buffer),
                    },
                ],
            });

        let bind_group_divide = self
            .bind_group_manager_divide
            .get_or_create(BindGroupDescriptor {
                entries: vec![
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::Buffer(working_buffer),
                    },
                    BindGroupEntry {
                        binding: 1,
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
            compute_pass.dispatch_workgroups(1, 1, 1);

            compute_pass.set_pipeline(&self.pipeline_sum);
            compute_pass.set_bind_group(0, &bind_group_sum, &[]);

            let num_workgroups_x = div_up(input_img.properties.dimensions.0, 16);
            let num_workgroups_y = div_up(input_img.properties.dimensions.1, 16);

            compute_pass.dispatch_workgroups(num_workgroups_x, num_workgroups_y, 1);

            compute_pass.set_pipeline(&self.pipeline_divide);
            compute_pass.set_bind_group(0, &bind_group_divide, &[]);
            compute_pass.dispatch_workgroups(1, 1, 1);
        }
    }
}
