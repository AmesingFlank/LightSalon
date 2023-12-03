use std::{collections::HashMap, mem::size_of, sync::Arc};

use crate::{
    buffer::{Buffer, BufferProperties, RingBuffer},
    engine::{toolbox::Toolbox, value_store::ValueStore},
    image::ColorSpace,
    ir::{ComputeGlobalMaskOp, Id},
    runtime::{
        BindGroupDescriptor, BindGroupDescriptorKey, BindGroupEntry, BindGroupManager,
        BindingResource, Runtime,
    },
    shader::{Shader, ShaderLibraryModule},
    utils::math::div_up,
};

pub struct ComputeGlobalMaskImpl {
    runtime: Arc<Runtime>,
    pipeline: wgpu::ComputePipeline,
    bind_group_manager: BindGroupManager,
}
impl ComputeGlobalMaskImpl {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let shader_code = Shader::from_code(include_str!("shaders/global_mask.wgsl"))
            .with_library(ShaderLibraryModule::ColorSpaces)
            .full_code();

        let (pipeline, bind_group_layout) = runtime.create_compute_pipeline(shader_code.as_str());

        let bind_group_manager = BindGroupManager::new(runtime.clone(), bind_group_layout);

        ComputeGlobalMaskImpl {
            runtime,
            pipeline,
            bind_group_manager,
        }
    }
}
impl ComputeGlobalMaskImpl {
    pub fn reset(&mut self) {}

    pub fn encode_commands(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        op: &ComputeGlobalMaskOp,
        value_store: &mut ValueStore,
        toolbox: &mut Toolbox,
    ) {
        let target_img = value_store.map.get(&op.target).unwrap().as_image().clone();
        let output_img = value_store.ensure_value_at_id_is_image_of_properties(
            self.runtime.as_ref(),
            op.result,
            &target_img.properties,
        );

        let bind_group = self.bind_group_manager.get_or_create(BindGroupDescriptor {
            entries: vec![BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureStorage(&output_img, 0),
            }],
        });

        {
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            compute_pass.set_pipeline(&self.pipeline);

            let num_workgroups_x = div_up(output_img.properties.dimensions.0, 16);
            let num_workgroups_y = div_up(output_img.properties.dimensions.1, 16);

            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(num_workgroups_x, num_workgroups_y, 1);
        }

        toolbox
            .mipmap_generator
            .encode_mipmap_generation_command(&output_img, encoder);
    }
}
