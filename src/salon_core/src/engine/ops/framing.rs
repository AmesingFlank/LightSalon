use std::{collections::HashMap, mem::size_of, sync::Arc};

use crate::runtime::Toolbox;

use crate::utils::math::get_cropped_image_dimensions;
use crate::{
    engine::value_store::ValueStore,
    ir::{ApplyFramingOp, Id},
    runtime::{
        BindGroupDescriptor, BindGroupDescriptorKey, BindGroupEntry, BindGroupManager,
        BindingResource, Runtime,
    },
    runtime::{Buffer, BufferProperties, RingBuffer, Sampler},
    runtime::{ColorSpace, ImageProperties},
    shader::{Shader, ShaderLibraryModule},
    utils::math::div_up,
};

pub struct ApplyFramingImpl {
    runtime: Arc<Runtime>,
    pipeline: wgpu::ComputePipeline,
    bind_group_manager: BindGroupManager,
}
impl ApplyFramingImpl {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let shader_code = Shader::from_code(include_str!("shaders/framing.wgsl")).full_code();

        let (pipeline, bind_group_layout) =
            runtime.create_compute_pipeline(shader_code.as_str(), Some("Framing"));

        let bind_group_manager = BindGroupManager::new(runtime.clone(), bind_group_layout);

        ApplyFramingImpl {
            runtime,
            pipeline,
            bind_group_manager,
        }
    }
}
impl ApplyFramingImpl {
    pub fn reset(&mut self) {
        self.bind_group_manager.clear_cache();
    }

    pub fn encode_commands(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        op: &ApplyFramingOp,
        value_store: &mut ValueStore,
        toolbox: &Toolbox,
    ) {
        let input_img = value_store.map.get(&op.arg).unwrap().as_image().clone();

        let input_dimensions = input_img.properties.dimensions;
        let output_aspect_ratio = op.frame.aspect_ratio_float();
        let output_dimensions = if input_img.aspect_ratio() >= output_aspect_ratio {
            let output_y = ((1.0 + op.frame.gap) * input_dimensions.1 as f32) as u32;
            let output_x = (output_y as f32 / output_aspect_ratio) as u32;
            (output_x, output_y)
        } else {
            let output_x = ((1.0 + op.frame.gap) * input_dimensions.0 as f32) as u32;
            let output_y = (output_x as f32 * output_aspect_ratio) as u32;
            (output_x, output_y)
        };

        let output_properties = ImageProperties {
            dimensions: output_dimensions,
            ..input_img.properties
        };

        let output_img = value_store.ensure_value_at_id_is_image_of_properties(
            self.runtime.as_ref(),
            op.result,
            &output_properties,
        );

        let bind_group = self.bind_group_manager.get_or_create(BindGroupDescriptor {
            entries: vec![
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Texture(&input_img),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureStorage(&output_img, 0),
                },
            ],
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                ..Default::default()
            });
            compute_pass.set_pipeline(&self.pipeline);

            let num_workgroups_x = div_up(output_dimensions.0, 16);
            let num_workgroups_y = div_up(output_dimensions.1, 16);

            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(num_workgroups_x, num_workgroups_y, 1);
        }
    }
}
