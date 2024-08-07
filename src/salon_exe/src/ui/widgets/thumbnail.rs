use std::mem::size_of;
use std::sync::Arc;
use std::{collections::HashMap};

use eframe::egui_wgpu::ScreenDescriptor;
use eframe::{egui, egui_wgpu};
use salon_core::runtime::Image;
use salon_core::runtime::Sampler;
use salon_core::runtime::{
    BindGroupDescriptor, BindGroupDescriptorKey, BindGroupEntry, BindGroupManager, BindingResource,
    Runtime,
};
use salon_core::runtime::{BufferProperties, RingBuffer};
use salon_core::shader::{Shader, ShaderLibraryModule};


pub struct ThumbnailCallback {
    pub image: Arc<Image>,
    pub allocated_ui_rect: egui::Rect,
    pub clip: ThumbnailClip,
}

pub enum ThumbnailClip {
    None,
    Top,
    Bottom,
}

impl egui_wgpu::CallbackTrait for ThumbnailCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &mut ThumbnailRenderResources = resources.get_mut().unwrap();
        resources.prepare(device, queue, self);
        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        resources: &'a egui_wgpu::CallbackResources,
    ) {
        let resources: &ThumbnailRenderResources = resources.get().unwrap();
        resources.paint(render_pass, self.image.as_ref());
    }
}

pub struct ThumbnailRenderResources {
    pipeline: wgpu::RenderPipeline,
    bind_group_manager: BindGroupManager,
    bind_group_key_cache: HashMap<u32, BindGroupDescriptorKey>, // image uuid -> key
    ring_buffer: RingBuffer,
    texture_sampler: Sampler,
}

impl ThumbnailRenderResources {
    pub fn new(runtime: Arc<Runtime>, target_format: wgpu::TextureFormat) -> Self {
        let shader_code = Shader::from_code(include_str!("../shaders/thumbnail.wgsl"))
            .with_library(ShaderLibraryModule::ColorSpaces)
            .full_code();

        let (pipeline, bind_group_layout) =
            runtime.create_render_pipeline(shader_code.as_str(), target_format, Some("Thumbnail"));

        let bind_group_manager = BindGroupManager::new(runtime.clone(), bind_group_layout);

        let ring_buffer = RingBuffer::new(
            runtime.clone(),
            BufferProperties {
                size: size_of::<u32>() + size_of::<f32>() * 2,
                host_readable: false,
            },
        );

        let texture_sampler = runtime.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        ThumbnailRenderResources {
            pipeline,
            bind_group_manager,
            bind_group_key_cache: HashMap::new(),
            ring_buffer,
            texture_sampler,
        }
    }

    pub fn reset(&mut self) {
        self.ring_buffer.mark_all_available();
    }

    fn prepare(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_call: &ThumbnailCallback,
    ) {
        let buffer = self.ring_buffer.get();
        queue.write_buffer(
            &buffer.buffer,
            0,
            bytemuck::cast_slice(&[render_call.image.properties.color_space as u32]),
        );

        let (mut min_v, mut max_v) = (0.0f32, 1.0f32);
        match render_call.clip {
            ThumbnailClip::Bottom => {
                max_v = render_call.image.aspect_ratio() / render_call.allocated_ui_rect.aspect_ratio();
                assert!(max_v <= 1.0);
            }
            ThumbnailClip::Top => {
                min_v = 1.0 - render_call.image.aspect_ratio() / render_call.allocated_ui_rect.aspect_ratio();
                assert!(min_v >= 0.0);
            }
            ThumbnailClip::None => {}
        }
        queue.write_buffer(
            &buffer.buffer,
            size_of::<u32>() as u64,
            bytemuck::cast_slice(&[min_v, max_v]),
        );

        let bind_group_desc = BindGroupDescriptor {
            entries: vec![
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(buffer),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Texture(&render_call.image),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&self.texture_sampler),
                },
            ],
        };
        let bind_group_key = bind_group_desc.to_key();
        self.bind_group_manager.ensure(bind_group_desc);
        self.bind_group_key_cache
            .insert(render_call.image.uuid, bind_group_key);
    }

    fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>, image: &'rp Image) {
        let bind_group_key = self.bind_group_key_cache.get(&image.uuid).unwrap();
        let bind_group = self
            .bind_group_manager
            .get_from_key_or_panic(bind_group_key);

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}
