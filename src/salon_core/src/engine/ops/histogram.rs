use std::{collections::HashMap, mem::size_of, sync::Arc};

use crate::{
    buffer::BufferProperties,
    engine::value_store::ValueStore,
    image::ColorSpace,
    ir::{ComputeHistogramOp, Id},
    runtime::{
        BindGroupDescriptor, BindGroupDescriptorKey, BindGroupEntry, BindGroupManager,
        BindingResource, Runtime,
    },
    shader::{Shader, ShaderLibraryModule},
};

pub struct ComputeHistogramImpl {
    runtime: Arc<Runtime>,

    pipeline_clear: wgpu::ComputePipeline,
    bind_group_manager_clear: BindGroupManager,

    pipeline_compute: wgpu::ComputePipeline,
    bind_group_manager_compute: BindGroupManager,
}
impl ComputeHistogramImpl {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let shader_clear = Shader::from_code(include_str!("shaders/histogram_clear.wgsl"))
            .with_library(ShaderLibraryModule::ColorSpaces)
            .full_code();
        let (pipeline_clear, bind_group_layout_clear) =
            runtime.create_compute_pipeline(shader_clear.as_str());
        let bind_group_manager_clear =
            BindGroupManager::new(runtime.clone(), bind_group_layout_clear);

        let shader_compute = Shader::from_code(include_str!("shaders/histogram_compute.wgsl"))
            .with_library(ShaderLibraryModule::ColorSpaces)
            .full_code();
        let (pipeline_compute, bind_group_layout_compute) =
            runtime.create_compute_pipeline(shader_compute.as_str());
        let bind_group_manager_compute =
            BindGroupManager::new(runtime.clone(), bind_group_layout_compute);

        ComputeHistogramImpl {
            runtime,
            pipeline_clear,
            bind_group_manager_clear,
            pipeline_compute,
            bind_group_manager_compute,
        }
    }
}
impl ComputeHistogramImpl {
    pub fn reset(&mut self) {}

    pub fn encode_commands(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        op: &ComputeHistogramOp,
        value_store: &mut ValueStore,
    ) {
        let input_img = value_store.map.get(&op.arg).unwrap().as_image().clone();

        let buffer_props = BufferProperties {
            size: 4 * 256 * size_of::<u32>(),
            host_readable: true,
        };

        let output_buffer = value_store.ensure_value_at_id_is_buffer_of_properties(
            self.runtime.as_ref(),
            op.result,
            &buffer_props,
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
                            resource: BindingResource::Buffer(&output_buffer),
                        },
                    ],
                });

        {
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });

            compute_pass.set_pipeline(&self.pipeline_clear);
            compute_pass.set_bind_group(0, &bind_group_clear, &[]);
            compute_pass.dispatch_workgroups(256, 1, 1);

            compute_pass.set_pipeline(&self.pipeline_compute);
            compute_pass.set_bind_group(0, &bind_group_compute, &[]);

            compute_pass.dispatch_workgroups(
                input_img.properties.dimensions.0,
                input_img.properties.dimensions.1,
                1,
            );
        }
    }
}
