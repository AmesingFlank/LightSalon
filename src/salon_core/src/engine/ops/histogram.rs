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
    pipeline: wgpu::ComputePipeline,
    bind_group_manager: BindGroupManager,
    bind_group_key_cache: HashMap<Id, BindGroupDescriptorKey>,
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
            bind_group_key_cache: HashMap::new(),
        }
    }
}
impl ComputeHistogramImpl {
    pub fn reset(&mut self) {}

    pub fn prepare(&mut self, op: &ComputeHistogramOp, value_store: &mut ValueStore) {
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

        self.runtime.queue.write_buffer(
            &output_buffer.buffer,
            0,
            bytemuck::cast_slice(&[0u32; 4 * 256]),
        );

        let bind_group_descriptor = BindGroupDescriptor {
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
        };
        let bind_group_key = bind_group_descriptor.to_key();
        self.bind_group_manager.ensure(bind_group_descriptor);
        self.bind_group_key_cache.insert(op.result, bind_group_key);
    }

    pub fn encode_commands<'a>(
        &'a self,
        encoder: &mut wgpu::CommandEncoder,
        op: &ComputeHistogramOp,
        value_store: &mut ValueStore,
    ) {
        let input_img = value_store.map.get(&op.arg).unwrap().as_image();
        let bind_group_key = self.bind_group_key_cache.get(&op.result).unwrap();
        let bind_group = self
            .bind_group_manager
            .get_from_key_or_panic(bind_group_key);

        {
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            compute_pass.set_pipeline(&self.pipeline);

            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(
                input_img.properties.dimensions.0,
                input_img.properties.dimensions.1,
                1,
            );
        }
    }
}
