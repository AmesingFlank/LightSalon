use crate::runtime::Runtime;

pub struct Buffer {
    pub properties: BufferProperties,
    pub buffer: wgpu::Buffer,
    pub uuid: u32,
}

#[derive(Clone, PartialEq, Eq)]
pub struct BufferProperties {
    pub size: usize,
}

pub struct RingBuffer {
    properties: BufferProperties,
    buffers: Vec<Buffer>,
    next_available_index: usize,
}

impl RingBuffer {
    pub fn new(properties: BufferProperties) -> Self {
        RingBuffer {
            properties,
            buffers: Vec::new(),
            next_available_index: 0,
        }
    }
    pub fn get(&mut self, runtime: &Runtime) -> &Buffer {
        while self.buffers.len() < self.next_available_index + 1 {
            let new_buffer = runtime.create_buffer_of_properties(self.properties.clone());
            self.buffers.push(new_buffer);
            assert!(self.buffers.len() < 100, "ring buffer size over 100! something is probably wrong");
        }
        let result = &self.buffers[self.next_available_index];
        self.next_available_index = self.next_available_index + 1;
        result
    }
    pub fn mark_all_available(&mut self) {
        self.next_available_index = 0;
    }
}
