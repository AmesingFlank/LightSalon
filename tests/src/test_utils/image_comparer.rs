use std::collections::HashMap;
use std::{mem::size_of, sync::Arc};

use salon_core::runtime::{
    BindGroupDescriptor, BindGroupEntry, BindGroupManager, BindingResource, Buffer,
    BufferProperties, BufferReader, Image, ImageFormat, Runtime,
};
use salon_core::shader::Shader;
use salon_core::utils::math::div_up;

pub struct ImageComparer {
    pipeline_clear: wgpu::ComputePipeline,
    bind_group_manager_clear: BindGroupManager,
    pipeline_sum: wgpu::ComputePipeline,
    bind_group_manager_sum: BindGroupManager,

    accumulation_buffer: Arc<Buffer>,
    runtime: Arc<Runtime>,
}
impl ImageComparer {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let accumulation_buffer = Arc::new(runtime.create_buffer_of_properties(BufferProperties {
            size: size_of::<u32>() * 2,
            host_readable: true,
        }));

        let shader_code = include_str!("./shaders/image_comparer_clear.wgsl");
        let shader_code = Shader::from_code(shader_code).full_code();

        let (pipeline_clear, bind_group_layout) =
            runtime.create_compute_pipeline(shader_code.as_str(), Some("ImageComparer"));
        let bind_group_manager_clear = BindGroupManager::new(runtime.clone(), bind_group_layout);

        let shader_code = include_str!("./shaders/image_comparer_sum.wgsl");
        let shader_code = Shader::from_code(shader_code).full_code();

        let (pipeline_sum, bind_group_layout) =
            runtime.create_compute_pipeline(shader_code.as_str(), Some("ImageComparer"));
        let bind_group_manager_sum = BindGroupManager::new(runtime.clone(), bind_group_layout);

        ImageComparer {
            runtime,
            pipeline_clear,
            bind_group_manager_clear,
            pipeline_sum,
            bind_group_manager_sum,
            accumulation_buffer,
        }
    }
}
impl ImageComparer {
    pub fn assert_eq(&mut self, lhs: Arc<Image>, rhs: Arc<Image>, rmse_threshold: f32) {
        assert_eq!(lhs.properties.dimensions, rhs.properties.dimensions);
        assert_eq!(lhs.properties.format, rhs.properties.format);

        let bind_group_clear = self
            .bind_group_manager_clear
            .get_or_create(BindGroupDescriptor {
                entries: vec![BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(&self.accumulation_buffer),
                }],
            });

        self.bind_group_manager_sum.clear_cache();
        let bind_group_sum = self
            .bind_group_manager_sum
            .get_or_create(BindGroupDescriptor {
                entries: vec![
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::Buffer(&self.accumulation_buffer),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Texture(&lhs),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::Texture(&rhs),
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
            cpass.set_pipeline(&self.pipeline_clear);
            cpass.set_bind_group(0, bind_group_clear, &[]);
            cpass.dispatch_workgroups(1, 1, 1);

            cpass.set_pipeline(&self.pipeline_sum);
            cpass.set_bind_group(0, bind_group_sum, &[]);
            let num_workgroups_x = div_up(lhs.properties.dimensions.0, 16);
            let num_workgroups_y = div_up(lhs.properties.dimensions.1, 16);
            cpass.dispatch_workgroups(num_workgroups_x, num_workgroups_y, 1);
        }
        self.runtime.queue.submit(Some(encoder.finish()));

        let mut buffer_reader = BufferReader::new(
            self.runtime.clone(),
            self.accumulation_buffer.clone(),
            None,
            Box::new(|x| x),
        );
        let buffer_values =
            futures::executor::block_on(async move { buffer_reader.await_value().await.clone() });

        // matches error_factor from image_comparer_sum.wgsl
        const ERROR_FACTOR: f32 = 255.0;
        let mse = (buffer_values[0] as f32 / ERROR_FACTOR) / buffer_values[1] as f32;
        let rmse = mse.sqrt();
        println!("rmse: {}", rmse);
        assert!(rmse <= rmse_threshold);
        assert!(false);
    }
}
