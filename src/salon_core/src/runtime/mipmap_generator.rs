use std::sync::Arc;

use crate::{image::Image, runtime::Runtime};

pub struct MipmapGenerator {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    runtime: Arc<Runtime>,
}

// https://github.com/gfx-rs/wgpu/blob/trunk/examples/mipmap/src/main.rs
impl MipmapGenerator {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let (pipeline, bind_group_layout) = runtime.create_render_pipeline(
            include_str!("./mipmap_generator.wgsl"),
            wgpu::TextureFormat::Rgba16Float,
        );
        let sampler = runtime.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        MipmapGenerator {
            pipeline,
            bind_group_layout,
            sampler,
            runtime,
        }
    }

    pub fn encode_mipmap_generation_command(
        &self,
        img: &Image,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mip_count = Image::mip_level_count(&img.properties.dimensions);

        for target_mip in 1..mip_count as usize {
            let bind_group = self
                .runtime
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &img.texture_view_single_mip[target_mip - 1],
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                    ],
                    label: None,
                });

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &img.texture_view_single_mip[target_mip],
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }
    }

    pub fn generate(&self, img: &Image) {
        let mut encoder = self
            .runtime
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.encode_mipmap_generation_command(img, &mut encoder);
        self.runtime.queue.submit(Some(encoder.finish()));
    }
}
