use std::sync::mpsc::Receiver;
use std::sync::Arc;

use crate::{ir::Op, runtime::Runtime};

pub struct Buffer {
    pub properties: BufferProperties,
    pub buffer: wgpu::Buffer,
    pub buffer_host_readable: Option<wgpu::Buffer>,
    pub uuid: u32,
}

#[derive(Clone, PartialEq, Eq)]
pub struct BufferProperties {
    pub size: usize,
    pub host_readable: bool,
}

pub struct RingBuffer {
    runtime: Arc<Runtime>,
    properties: BufferProperties,
    buffers: Vec<Buffer>,
    next_available_index: usize,
}

impl RingBuffer {
    pub fn new(runtime: Arc<Runtime>, properties: BufferProperties) -> Self {
        RingBuffer {
            runtime,
            properties,
            buffers: Vec::new(),
            next_available_index: 0,
        }
    }
    pub fn get(&mut self) -> &Buffer {
        while self.buffers.len() < self.next_available_index + 1 {
            let new_buffer = self
                .runtime
                .create_buffer_of_properties(self.properties.clone());
            self.buffers.push(new_buffer);
            assert!(
                self.buffers.len() < 100,
                "ring buffer size over 100! something is probably wrong"
            );
        }
        let result = &self.buffers[self.next_available_index];
        self.next_available_index = self.next_available_index + 1;
        result
    }
    pub fn mark_all_available(&mut self) {
        self.next_available_index = 0;
    }
}

pub struct BufferReader<ValueType> {
    runtime: Arc<Runtime>,
    buffer: Arc<Buffer>,
    map_ready_receiver: Receiver<()>,
    transform: Box<dyn FnOnce(Vec<u32>) -> ValueType>,
    value: Option<ValueType>,
    pending_read: bool,
}

impl<ValueType> BufferReader<ValueType> {
    pub fn new(
        runtime: Arc<Runtime>,
        buffer: Arc<Buffer>,
        initial_value: Option<ValueType>,
        transform: Box<dyn FnOnce(Vec<u32>) -> ValueType>,
    ) -> Self {
        let map_ready_receiver = runtime.map_host_readable_buffer(&buffer);
        Self {
            runtime,
            buffer,
            map_ready_receiver,
            transform,
            value: initial_value,
            pending_read: true
        }
    }

    pub fn poll(&mut self) {
        if let Ok(_) = self.map_ready_receiver.try_recv() {
            let data: Vec<u32> = self.runtime.read_mapped_buffer(&self.buffer);
            let transform = std::mem::replace(
                &mut self.transform,
                Box::new(|_| panic!("Function called more than once")),
            );
            self.value = Some(transform(data));
            self.pending_read = false;
        }
    }

    pub fn value_ref(&self) -> Option<&ValueType> {
        self.value.as_ref()
    }

    pub fn take_value(&mut self) -> Option<ValueType> {
        self.value.take()
    }

    pub fn pending_read(&self) -> bool {
        self.pending_read
    }

    pub fn buffer(&self) -> &Arc<Buffer> {
        &self.buffer
    }
}
