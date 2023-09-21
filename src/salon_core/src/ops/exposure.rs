use std::{collections::HashMap, sync::Arc};

use crate::{image::Image, runtime::Runtime};

pub struct ExposureOp {
    runtime: Arc<Runtime>,
    resources: ExposureOpResources,
}

struct ExposureOpResources {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_groups: HashMap<(u32, u32), wgpu::BindGroup>,
}

impl ExposureOp {
    fn new(runtime: Arc<Runtime>) -> Self {
        let shader = runtime
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("./exposure.wgsl").into()),
            });

        let bind_group_layout =
            runtime
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::ReadOnly,
                                format: wgpu::TextureFormat::Rgba8Unorm,
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::ReadWrite,
                                format: wgpu::TextureFormat::Rgba8Unorm,
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                    ],
                });

        let pipeline_layout =
            runtime
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });

        let pipeline = runtime
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: "cs_main",
            });

        let resources = ExposureOpResources {
            pipeline,
            bind_group_layout,
            bind_groups: HashMap::new(),
        };
        ExposureOp {
            runtime: runtime,
            resources,
        }
    }

    fn apply<'img>(
        &'img mut self,
        input: &'img Image,
        output: &'img Image,
        from_value: f32,
        to_value: f32,
    ) {
    }
}
