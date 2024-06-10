use std::{collections::HashMap, mem::size_of, sync::Arc};

use crate::runtime::Toolbox;

use crate::{
    engine::{value_store::ValueStore},
    ir::{ApplyMaskedEditsOp, Id},
    runtime::{
        BindGroupDescriptor, BindGroupDescriptorKey, BindGroupEntry, BindGroupManager,
        BindingResource, Runtime,
    },
    runtime::{Buffer, BufferProperties, RingBuffer},
    shader::{Shader, ShaderLibraryModule},
    utils::math::div_up,
};

pub struct ApplyMaskedEditsImpl {
    runtime: Arc<Runtime>,
    pipeline: wgpu::ComputePipeline,
    bind_group_manager: BindGroupManager,
}
impl ApplyMaskedEditsImpl {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let shader_code = Shader::from_code(include_str!("shaders/apply_masked_edits.wgsl"))
            .with_library(ShaderLibraryModule::ColorSpaces)
            .full_code();

        let (pipeline, bind_group_layout) =
            runtime.create_compute_pipeline(shader_code.as_str(), Some("ApplyMaskedEdits"));

        let bind_group_manager = BindGroupManager::new(runtime.clone(), bind_group_layout);

        ApplyMaskedEditsImpl {
            runtime,
            pipeline,
            bind_group_manager,
        }
    }
}
impl ApplyMaskedEditsImpl {
    pub fn reset(&mut self) {
        self.bind_group_manager.clear_cache();
    }

    pub fn encode_commands(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        op: &ApplyMaskedEditsOp,
        value_store: &mut ValueStore,
        toolbox: &Toolbox,
    ) {
        let original_img = value_store
            .map
            .get(&op.original_target)
            .unwrap()
            .as_image()
            .clone();
        let edited_img = value_store.map.get(&op.edited).unwrap().as_image().clone();
        let mask_img = value_store.map.get(&op.mask).unwrap().as_image().clone();

        let output_img = value_store.ensure_value_at_id_is_image_of_properties(
            self.runtime.as_ref(),
            op.result,
            &edited_img.properties,
        );

        let bind_group = self.bind_group_manager.get_or_create(BindGroupDescriptor {
            entries: vec![
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Texture(&original_img),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Texture(&edited_img),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Texture(&mask_img),
                },
                BindGroupEntry {
                    binding: 3,
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
    }
}
