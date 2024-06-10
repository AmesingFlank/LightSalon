use std::collections::HashMap;
use std::{mem::size_of, sync::Arc};

use crate::runtime::{Buffer, BufferProperties, ColorSpace, Image, ImageFormat, Runtime};

use crate::shader::{Shader, ShaderLibraryModule};
use crate::utils::math::div_up;

use super::{
    bind_group_manager, BindGroupDescriptor, BindGroupEntry, BindGroupManager, BindingResource,
};

pub struct ImageFormatConverter {
    pipelines: HashMap<ImageFormat, (wgpu::ComputePipeline, BindGroupManager)>,
    runtime: Arc<Runtime>,
}
impl ImageFormatConverter {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        ImageFormatConverter {
            runtime,
            pipelines: HashMap::new(),
        }
    }
}
impl ImageFormatConverter {
    pub fn convert(&mut self, input_img: Arc<Image>, dest_image_format: ImageFormat) -> Arc<Image> {
        if input_img.properties.format == dest_image_format {
            return input_img;
        }

        if !self.pipelines.contains_key(&dest_image_format) {
            let shader_code = include_str!("./image_format_converter.wgsl")
                .replace("IMAGE_FORMAT", dest_image_format.to_wgsl_format_string());
            let shader_code = Shader::from_code(shader_code.as_str())
                .with_library(ShaderLibraryModule::ColorSpaces)
                .full_code();

            let (pipeline, bind_group_layout) = self
                .runtime
                .create_compute_pipeline(shader_code.as_str(), Some("ImageFormatConverter"));
            let bind_group_manager = BindGroupManager::new(self.runtime.clone(), bind_group_layout);

            self.pipelines
                .insert(dest_image_format, (pipeline, bind_group_manager));
        }

        let (pipeline, bind_group_manager) = self.pipelines.get_mut(&dest_image_format).unwrap();

        bind_group_manager.clear_cache();

        let mut properties = input_img.properties.clone();
        properties.format = dest_image_format;

        let output_img = Arc::new(self.runtime.create_image_of_properties(properties));

        let bind_group = bind_group_manager.get_or_create(BindGroupDescriptor {
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

        let mut encoder = self
            .runtime
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                ..Default::default()
            });
            cpass.set_pipeline(pipeline);

            cpass.set_bind_group(0, &bind_group, &[]);

            let num_workgroups_x = div_up(input_img.properties.dimensions.0, 16);
            let num_workgroups_y = div_up(input_img.properties.dimensions.1, 16);
            cpass.dispatch_workgroups(num_workgroups_x, num_workgroups_y, 1);
        }
        self.runtime.queue.submit(Some(encoder.finish()));
        output_img
    }
}
