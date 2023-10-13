use std::{collections::HashMap, mem::size_of, sync::Arc};

use crate::{
    buffer::{Buffer, BufferProperties, RingBuffer},
    engine::value_store::ValueStore,
    image::ColorSpace,
    ir::{AdjustSaturationOp, Id},
    runtime::{
        BindGroupDescriptor, BindGroupDescriptorKey, BindGroupEntry, BindGroupManager,
        BindingResource, Runtime,
    },
    shader::{Shader, ShaderLibraryModule},
};

pub struct AdjustSaturationImpl {
    runtime: Arc<Runtime>,
    pipeline: wgpu::ComputePipeline,
    bind_group_manager: BindGroupManager,
    bind_group_key_cache: HashMap<Id, BindGroupDescriptorKey>,
    ring_buffer: RingBuffer,
}
impl AdjustSaturationImpl {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let shader_code = Shader::from_code(include_str!("./saturation.wgsl"))
            .with_library(ShaderLibraryModule::ColorSpaces)
            .full_code();

        let (pipeline, bind_group_layout) = runtime.create_compute_pipeline(shader_code.as_str());

        let bind_group_manager = BindGroupManager::new(runtime.clone(), bind_group_layout);

        let ring_buffer = RingBuffer::new(
            runtime.clone(),
            BufferProperties {
                size: size_of::<f32>(),
                host_readable: false,
            },
        );

        AdjustSaturationImpl {
            runtime,
            pipeline,
            bind_group_manager,
            bind_group_key_cache: HashMap::new(),
            ring_buffer,
        }
    }
}
impl AdjustSaturationImpl {
    pub fn reset(&mut self) {
        self.ring_buffer.mark_all_available();
    }

    pub fn prepare(&mut self, op: &AdjustSaturationOp, value_store: &mut ValueStore) {
        let input_img = value_store.map.get(&op.arg).unwrap().as_image().clone();
        let output_img = value_store.ensure_value_at_id_is_image_of_properties(
            self.runtime.as_ref(),
            op.result,
            &input_img.properties,
        );

        let buffer = self.ring_buffer.get();

        self.runtime
            .queue
            .write_buffer(&buffer.buffer, 0, bytemuck::cast_slice(&[op.saturation]));

        let bind_group_descriptor = BindGroupDescriptor {
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
        };
        let bind_group_key = bind_group_descriptor.to_key();
        self.bind_group_manager.ensure(bind_group_descriptor);
        self.bind_group_key_cache.insert(op.result, bind_group_key);
    }

    pub fn encode_commands<'a>(
        &'a self,
        compute_pass: &mut wgpu::ComputePass<'a>,
        op: &AdjustSaturationOp,
        value_store: &ValueStore,
    ) {
        let input_img = value_store.map.get(&op.arg).unwrap().as_image();
        let bind_group_key = self.bind_group_key_cache.get(&op.result).unwrap();
        let bind_group = self
            .bind_group_manager
            .get_from_key_or_panic(bind_group_key);

        compute_pass.set_pipeline(&self.pipeline);

        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(
            input_img.properties.dimensions.0,
            input_img.properties.dimensions.1,
            1,
        );
    }
}
