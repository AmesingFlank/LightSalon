use std::{collections::HashMap, mem::size_of, sync::Arc};

use crate::{
    buffer::{Buffer, BufferProperties, RingBuffer},
    engine::value_store::ValueStore,
    image::ColorSpace,
    ir::{Id, PrepareDehazeOp},
    runtime::{
        BindGroupDescriptor, BindGroupDescriptorKey, BindGroupEntry, BindGroupManager,
        BindingResource, Runtime,
    },
    shader::{Shader, ShaderLibraryModule},
    utils::math::div_up,
};

pub struct PrepareDehazeImpl {
    runtime: Arc<Runtime>,

    ring_buffer: RingBuffer,

    pipeline_clear_histogram: wgpu::ComputePipeline,
    bind_group_manager_clear_histogram: BindGroupManager,

    pipeline_compute_histogram: wgpu::ComputePipeline,
    bind_group_manager_compute_histogram: BindGroupManager,

    pipeline_estimate_airlight: wgpu::ComputePipeline,
    bind_group_manager_estimate_airlight: BindGroupManager,

    pipeline_prepare: wgpu::ComputePipeline,
    bind_group_manager_prepare: BindGroupManager,
}

const NUM_BINS: i32 = 256;

impl PrepareDehazeImpl {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let shader_code =
            Shader::from_code(include_str!("shaders/dehaze_clear_histogram.wgsl")).full_code();
        let (pipeline_clear_histogram, bind_group_layout) =
            runtime.create_compute_pipeline(shader_code.as_str());
        let bind_group_manager_clear_histogram =
            BindGroupManager::new(runtime.clone(), bind_group_layout);

        let shader_code =
            Shader::from_code(include_str!("shaders/dehaze_compute_histogram.wgsl")).full_code();
        let (pipeline_compute_histogram, bind_group_layout) =
            runtime.create_compute_pipeline(shader_code.as_str());
        let bind_group_manager_compute_histogram =
            BindGroupManager::new(runtime.clone(), bind_group_layout);

        let shader_code =
            Shader::from_code(include_str!("shaders/dehaze_estimate_airlight.wgsl")).full_code();
        let (pipeline_estimate_airlight, bind_group_layout) =
            runtime.create_compute_pipeline(shader_code.as_str());
        let bind_group_manager_estimate_airlight =
            BindGroupManager::new(runtime.clone(), bind_group_layout);

        let shader_code = Shader::from_code(include_str!("shaders/dehaze_prepare.wgsl"))
            .with_library(ShaderLibraryModule::ColorSpaces)
            .full_code();
        let (pipeline_prepare, bind_group_layout) =
            runtime.create_compute_pipeline(shader_code.as_str());
        let bind_group_manager_prepare = BindGroupManager::new(runtime.clone(), bind_group_layout);

        let ring_buffer = RingBuffer::new(
            runtime.clone(),
            BufferProperties {
                size: size_of::<f32>() + size_of::<u32>() * NUM_BINS as usize,
                host_readable: false,
            },
        );

        Self {
            runtime,
            ring_buffer,

            pipeline_prepare,
            bind_group_manager_prepare,

            pipeline_clear_histogram,
            bind_group_manager_clear_histogram,

            pipeline_compute_histogram,
            bind_group_manager_compute_histogram,

            pipeline_estimate_airlight,
            bind_group_manager_estimate_airlight,
        }
    }
}
impl PrepareDehazeImpl {
    pub fn reset(&mut self) {
        self.ring_buffer.mark_all_available();
    }

    pub fn encode_commands(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        op: &PrepareDehazeOp,
        value_store: &mut ValueStore,
    ) {
        let input_img = value_store.map.get(&op.arg).unwrap().as_image().clone();
        let output_img = value_store.ensure_value_at_id_is_image_of_properties(
            self.runtime.as_ref(),
            op.result,
            &input_img.properties,
        );

        let buffer = self.ring_buffer.get();

        let bind_group_clear_histogram =
            self.bind_group_manager_clear_histogram
                .get_or_create(BindGroupDescriptor {
                    entries: vec![BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::Buffer(buffer),
                    }],
                });

        let bind_group_compute_histogram =
            self.bind_group_manager_compute_histogram
                .get_or_create(BindGroupDescriptor {
                    entries: vec![
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::Texture(&input_img),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::Buffer(buffer),
                        },
                    ],
                });

        let bind_group_estimate_airlight =
            self.bind_group_manager_estimate_airlight
                .get_or_create(BindGroupDescriptor {
                    entries: vec![
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::Texture(&input_img),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::Buffer(buffer),
                        },
                    ],
                });

        let bind_group_prepare =
            self.bind_group_manager_prepare
                .get_or_create(BindGroupDescriptor {
                    entries: vec![
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::Texture(&input_img),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::TextureStorage(&output_img, 0),
                        },
                        // BindGroupEntry {
                        //     binding: 2,
                        //     resource: BindingResource::Buffer(buffer),
                        // },
                    ],
                });

        {
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });

            compute_pass.set_pipeline(&self.pipeline_clear_histogram);
            compute_pass.set_bind_group(0, &bind_group_clear_histogram, &[]);
            compute_pass.dispatch_workgroups(1, 1, 1);

            let num_workgroups_x = div_up(input_img.properties.dimensions.0, 8);
            let num_workgroups_y = div_up(input_img.properties.dimensions.1, 8);
            compute_pass.set_pipeline(&self.pipeline_prepare);
            compute_pass.set_bind_group(0, &bind_group_prepare, &[]);
            compute_pass.dispatch_workgroups(num_workgroups_x, num_workgroups_y, 1);
        }
    }
}
