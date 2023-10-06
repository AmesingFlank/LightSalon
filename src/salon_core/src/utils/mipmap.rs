use crate::{image::Image, runtime::Runtime};

pub struct MipmapGenerator {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

// https://github.com/gfx-rs/wgpu/blob/trunk/examples/mipmap/src/main.rs
impl MipmapGenerator {
    pub fn new(runtime: &Runtime) -> Self {
        let (pipeline, bind_group_layout) = runtime
            .create_render_pipeline(include_str!("./blit.wgsl"), wgpu::TextureFormat::Rgba8Unorm);
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
        }
    }

    pub fn encode_mipmap_generation_command<'i>(
        &'i self,
        runtime: &'i Runtime,
        img: &'i Image,
        encoder: &'i mut wgpu::CommandEncoder,
    ) {
        let mip_count = Image::mip_level_count(&img.properties.dimensions);
        let views = (0..mip_count)
            .map(|mip| {
                img.texture.create_view(&wgpu::TextureViewDescriptor {
                    label: None,
                    format: None,
                    dimension: None,
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: mip,
                    mip_level_count: Some(1),
                    base_array_layer: 0,
                    array_layer_count: None,
                })
            })
            .collect::<Vec<_>>();

        for target_mip in 1..mip_count as usize {
            let bind_group = runtime
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&views[target_mip - 1]),
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
                    view: &views[target_mip],
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

    pub fn generate<'i>(&'i self, runtime: &'i Runtime, img: &'i Image) {
        let mut encoder = runtime
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.encode_mipmap_generation_command(runtime, img, &mut encoder);
        runtime.queue.submit(Some(encoder.finish()));
    }
}
