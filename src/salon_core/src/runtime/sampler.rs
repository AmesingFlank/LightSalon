use crate::utils::uuid::Uuid;

pub struct Sampler {
    pub sampler: wgpu::Sampler,
    pub uuid: Uuid
}