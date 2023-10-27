use std::{collections::HashMap, mem::size_of, sync::Arc};

use crate::{
    buffer::{Buffer, BufferProperties, RingBuffer},
    engine::value_store::ValueStore,
    image::ColorSpace,
    ir::{CollectDataForEditorOp, Id},
    runtime::{
        BindGroupDescriptor, BindGroupDescriptorKey, BindGroupEntry, BindGroupManager,
        BindingResource, Runtime,
    },
    shader::{Shader, ShaderLibraryModule},
};

pub struct CollectDataForEditorImpl {
    runtime: Arc<Runtime>,
}
impl CollectDataForEditorImpl {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        CollectDataForEditorImpl { runtime }
    }
}
impl CollectDataForEditorImpl {
    pub fn reset(&mut self) {}

    pub fn encode_commands(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        op: &CollectDataForEditorOp,
        value_store: &mut ValueStore,
    ) {
        let histogram_buffer = value_store
            .map
            .get(&op.histogram_final)
            .unwrap()
            .as_buffer()
            .clone();
        let result_properties = BufferProperties {
            size: histogram_buffer.properties.size,
            host_readable: true,
        };
        let result_buffer = value_store.ensure_value_at_id_is_buffer_of_properties(
            self.runtime.as_ref(),
            op.result,
            &result_properties,
        );
        encoder.copy_buffer_to_buffer(
            &histogram_buffer.buffer,
            0,
            &result_buffer.buffer,
            0,
            histogram_buffer.properties.size as u64,
        );
    }
}
