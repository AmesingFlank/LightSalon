use std::{mem::size_of, sync::Arc};

use crate::{
    buffer::{BufferProperties, Buffer},
    image::{ColorSpace, Image},
    runtime::Runtime,
    shader::{Shader, ShaderLibraryModule},
};

use super::{
    bind_group_manager, BindGroupDescriptor, BindGroupEntry, BindGroupManager, BindingResource,
};

pub struct ColorSpaceConverter {
    pipeline: wgpu::ComputePipeline,
    bind_group_manager: BindGroupManager,
    uniform_buffer: Buffer,
    runtime: Arc<Runtime>,
}
impl ColorSpaceConverter {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let shader_code = Shader::from_code(include_str!("./color_space_converter.wgsl"))
            .with_library(ShaderLibraryModule::ColorSpaces)
            .full_code();

        let (pipeline, bind_group_layout) = runtime.create_compute_pipeline(shader_code.as_str());
        let bind_group_manager = BindGroupManager::new(runtime.clone(), bind_group_layout);

        let uniform_buffer = runtime.create_buffer_of_properties(BufferProperties {
            size: size_of::<u32>() * 2,
            host_readable: false,
        });

        ColorSpaceConverter {
            runtime,
            pipeline,
            bind_group_manager,
            uniform_buffer,
        }
    }
}
impl ColorSpaceConverter {
    pub fn convert(&mut self, input_img: Arc<Image>, dest_color_space: ColorSpace) -> Arc<Image> {
        if input_img.properties.color_space == dest_color_space {
            return input_img;
        }

        let mut properties = input_img.properties.clone();
        properties.color_space = dest_color_space;

        let output_img = Arc::new(self.runtime.create_image_of_properties(properties));

        let src_color_space = input_img.properties.color_space as u32;
        let dest_color_space = dest_color_space as u32;

        self.runtime.queue.write_buffer(
            &self.uniform_buffer.buffer,
            0,
            bytemuck::cast_slice(&[src_color_space, dest_color_space]),
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
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Buffer(&self.uniform_buffer),
                },
            ],
        });

        let mut encoder = self
            .runtime
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.pipeline);

            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(
                input_img.properties.dimensions.0,
                input_img.properties.dimensions.1,
                1,
            );
        }
        self.runtime.queue.submit(Some(encoder.finish()));
        output_img
    }
}
