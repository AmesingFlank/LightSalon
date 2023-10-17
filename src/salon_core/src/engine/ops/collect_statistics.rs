use std::{collections::HashMap, mem::size_of, sync::Arc};

use crate::{
    buffer::{Buffer, BufferProperties, RingBuffer},
    engine::value_store::ValueStore,
    image::ColorSpace,
    ir::{CollectStatisticsOp, Id},
    runtime::{
        BindGroupDescriptor, BindGroupDescriptorKey, BindGroupEntry, BindGroupManager,
        BindingResource, Runtime,
    },
    shader::{Shader, ShaderLibraryModule},
};

pub struct CollectStatisticsImpl {
    runtime: Arc<Runtime>,
}
impl CollectStatisticsImpl {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        CollectStatisticsImpl { runtime }
    }
}
impl CollectStatisticsImpl {
    pub fn reset(&mut self) {}

    pub fn prepare(&mut self, op: &CollectStatisticsOp, value_store: &mut ValueStore) {
        let histogram_buffer = value_store.map.get(&op.histogram).unwrap().as_buffer();
        let result_properties = BufferProperties {
            size: histogram_buffer.properties.size,
            host_readable: true,
        };
        value_store.ensure_value_at_id_is_buffer_of_properties(
            self.runtime.as_ref(),
            op.result,
            &result_properties,
        );
    }

    pub fn encode_commands<'a>(
        &'a self,
        compute_pass: &mut wgpu::ComputePass<'a>,
        op: &CollectStatisticsOp,
        value_store: &ValueStore,
    ) {
        // TODO impl
    }
}
