use std::collections::HashMap;
use std::{mem::size_of, sync::Arc};

use crate::runtime::{Buffer, BufferProperties, ColorSpace, Image, ImageFormat, Runtime};

use crate::shader::{Shader, ShaderLibraryModule};
use crate::utils::math::div_up;

use super::{
    bind_group_manager, BindGroupDescriptor, BindGroupEntry, BindGroupManager, BindingResource,
};

// do image->buffer copy using a compute shader to avoid having to deal with wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
pub struct ImageToBufferCopier {
    pipeline: wgpu::ComputePipeline,
    bind_group_manager: BindGroupManager,
    uniform_buffer: Buffer,
    runtime: Arc<Runtime>,
}
impl ImageToBufferCopier {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let uniform_buffer = runtime.create_buffer_of_properties(BufferProperties {
            size: size_of::<u32>() * 2,
            host_readable: false,
        });

        let shader_code =
            Shader::from_code(include_str!("./image_to_buffer_copier.wgsl")).full_code();

        let (pipeline, bind_group_layout) =
            runtime.create_compute_pipeline(shader_code.as_str(), Some("ImageToBufferCopier"));
        let bind_group_manager = BindGroupManager::new(runtime.clone(), bind_group_layout);

        ImageToBufferCopier {
            runtime,
            pipeline,
            bind_group_manager,
            uniform_buffer,
        }
    }
}
impl ImageToBufferCopier {
    pub fn copy(&mut self, input_img: &Image) -> Arc<Buffer> {
        assert!(
            input_img.properties.format == ImageFormat::Rgba8Unorm,
            "ImageToBufferCopier only handles rgba8unorm images"
        );
        let w = input_img.properties.dimensions.0;
        let h = input_img.properties.dimensions.1;
        let num_pixels = w * h;

        let buffer_properties = BufferProperties {
            size: (num_pixels * input_img.properties.format.bytes_per_pixel()) as usize,
            host_readable: true,
        };

        let output_buffer = Arc::new(self.runtime.create_buffer_of_properties(buffer_properties));

        self.runtime.queue.write_buffer(
            &self.uniform_buffer.buffer,
            0,
            bytemuck::cast_slice(&[w, h]),
        );

        self.bind_group_manager.clear_cache();

        let bind_group = self.bind_group_manager.get_or_create(BindGroupDescriptor {
            entries: vec![
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Texture(&input_img),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Buffer(&output_buffer),
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
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                ..Default::default()
            });
            cpass.set_pipeline(&self.pipeline);

            cpass.set_bind_group(0, &bind_group, &[]);

            let num_workgroups_x = div_up(input_img.properties.dimensions.0, 16);
            let num_workgroups_y = div_up(input_img.properties.dimensions.1, 16);
            cpass.dispatch_workgroups(num_workgroups_x, num_workgroups_y, 1);
        }
        self.runtime.queue.submit(Some(encoder.finish()));
        output_buffer
    }
}
