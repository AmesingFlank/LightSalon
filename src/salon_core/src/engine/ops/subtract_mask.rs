use std::{sync::Arc};

use crate::runtime::Toolbox;

use crate::{
    engine::value_store::ValueStore,
    ir::{SubtractMaskOp},
    runtime::{
        BindGroupDescriptor, BindGroupEntry, BindGroupManager,
        BindingResource, Runtime,
    },
    shader::{Shader},
    utils::math::div_up,
};

pub struct SubtractMaskImpl {
    runtime: Arc<Runtime>,
    pipeline: wgpu::ComputePipeline,
    bind_group_manager: BindGroupManager,
}
impl SubtractMaskImpl {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let shader_code = Shader::from_code(include_str!("shaders/subtract_mask.wgsl")).full_code();

        let (pipeline, bind_group_layout) =
            runtime.create_compute_pipeline(shader_code.as_str(), Some("SubtractMask"));

        let bind_group_manager = BindGroupManager::new(runtime.clone(), bind_group_layout);

        SubtractMaskImpl {
            runtime,
            pipeline,
            bind_group_manager,
        }
    }
}
impl SubtractMaskImpl {
    pub fn reset(&mut self) {
        self.bind_group_manager.clear_cache();
    }

    pub fn encode_commands(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        op: &SubtractMaskOp,
        value_store: &mut ValueStore,
        toolbox: &Toolbox,
    ) {
        let mask_0 = value_store.map.get(&op.mask_0).unwrap().as_image().clone();
        let mask_1 = value_store.map.get(&op.mask_1).unwrap().as_image().clone();

        let output_img = value_store.ensure_value_at_id_is_image_of_properties(
            self.runtime.as_ref(),
            op.result,
            &mask_0.properties,
        );

        let bind_group = self.bind_group_manager.get_or_create(BindGroupDescriptor {
            entries: vec![
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Texture(&mask_0),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Texture(&mask_1),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureStorage(&output_img, 0),
                },
            ],
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                ..Default::default()
            });
            compute_pass.set_pipeline(&self.pipeline);

            let num_workgroups_x = div_up(output_img.properties.dimensions.0, 16);
            let num_workgroups_y = div_up(output_img.properties.dimensions.1, 16);

            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(num_workgroups_x, num_workgroups_y, 1);
        }

        toolbox.encode_mipmap_generation_command(&output_img, encoder);
    }
}
