use std::{collections::HashMap, sync::Arc};

use crate::{
    image::{Image, ImageFormat},
    runtime::Runtime,
    sampler::Sampler,
};

use super::{BindGroupDescriptor, BindGroupEntry, BindGroupManager, BindingResource};

pub struct MipmapGenerator {
    pipelines: HashMap<ImageFormat, wgpu::RenderPipeline>,
    bind_group_manager: Option<BindGroupManager>,
    sampler: Sampler,
    runtime: Arc<Runtime>,
}

// https://github.com/gfx-rs/wgpu/blob/trunk/examples/mipmap/src/main.rs
impl MipmapGenerator {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let sampler = runtime.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        MipmapGenerator {
            pipelines: HashMap::new(),
            bind_group_manager: None,
            sampler,
            runtime,
        }
    }

    pub fn encode_mipmap_generation_command(
        &mut self,
        img: &Image,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        if !self.pipelines.contains_key(&img.properties.format) {
            let (pipeline, bind_group_layout) = self.runtime.create_render_pipeline(
                include_str!("./mipmap_generator.wgsl"),
                img.properties.format.to_wgpu_texture_format(),
            );
            self.pipelines.insert(img.properties.format, pipeline);
            if self.bind_group_manager.is_none() {
                self.bind_group_manager = Some(BindGroupManager::new(
                    self.runtime.clone(),
                    bind_group_layout,
                ));
            }
        }
        let pipeline = self.pipelines.get(&img.properties.format).unwrap();
        let mip_count = Image::mip_level_count(&img.properties.dimensions);

        for target_mip in 1..mip_count as usize {
            let bind_group =
                self.bind_group_manager
                    .as_mut()
                    .unwrap()
                    .get_or_create(BindGroupDescriptor {
                        entries: vec![
                            BindGroupEntry {
                                binding: 0,
                                resource: BindingResource::TextureSingleMip(
                                    &img,
                                    target_mip as u32 - 1,
                                ),
                            },
                            BindGroupEntry {
                                binding: 1,
                                resource: BindingResource::Sampler(&self.sampler),
                            },
                        ],
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
            rpass.set_pipeline(pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }
    }

    pub fn generate(&mut self, img: &Image) {
        let mut encoder = self
            .runtime
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.encode_mipmap_generation_command(img, &mut encoder);
        self.runtime.queue.submit(Some(encoder.finish()));
    }
}
