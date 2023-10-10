use std::{mem::size_of, sync::Arc};

use crate::{
    image::{ColorSpace, Image},
    runtime::Runtime,
    shader::{Shader, ShaderLibraryModule},
};

pub struct ColorSpaceConverter {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
}
impl ColorSpaceConverter {
    pub fn new(runtime: &Runtime) -> Self {
        let shader_code = Shader::from_code(include_str!("./color_space_converter.wgsl"))
            .with_library(ShaderLibraryModule::ColorSpaces)
            .full_code();

        let (pipeline, bind_group_layout) = runtime.create_compute_pipeline(shader_code.as_str());

        let uniform_buffer = runtime.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (size_of::<u32>() * 2) as u64,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        ColorSpaceConverter {
            pipeline,
            bind_group_layout,
            uniform_buffer,
        }
    }
}
impl ColorSpaceConverter {
    pub fn convert(
        &self,
        runtime: &Runtime,
        input_img: Arc<Image>,
        dest_color_space: ColorSpace,
    ) -> Arc<Image> {
        if input_img.properties.color_space == dest_color_space {
            return input_img;
        }

        let mut properties = input_img.properties.clone();
        properties.color_space = dest_color_space;

        let output_img = Arc::new(runtime.create_image_of_properties(properties));

        let src_color_space = input_img.properties.color_space as u32;
        let dest_color_space = dest_color_space as u32;

        runtime.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[src_color_space, dest_color_space]),
        );

        let bind_group = runtime
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&input_img.texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &output_img.texture_view_base_mip,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                ],
            });

        let mut encoder = runtime
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
        runtime.encode_mipmap_generation_command(output_img.as_ref(), &mut encoder);
        runtime.queue.submit(Some(encoder.finish()));
        output_img
    }
}
