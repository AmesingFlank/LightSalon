pub struct Buffer {
    pub properties: BufferProperties,
    pub buffer: wgpu::Buffer,
    pub uuid: u32,
}

#[derive(Clone, PartialEq, Eq)]
pub struct BufferProperties {
    pub size: usize,
}
