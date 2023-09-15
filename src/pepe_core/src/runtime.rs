use std::sync::Arc;

pub struct Runtime {
    pub adapter: Arc<wgpu::Adapter>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
}