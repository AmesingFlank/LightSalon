use std::collections::HashMap;
use std::sync::Arc;

use eframe::egui_wgpu::ScreenDescriptor;
use eframe::{egui, egui_wgpu};
use salon_core::runtime::Image;
use salon_core::runtime::Sampler;
use salon_core::runtime::{
    BindGroupDescriptor, BindGroupDescriptorKey, BindGroupEntry, BindGroupManager, BindingResource,
    Runtime,
};
use salon_core::shader::{Shader, ShaderLibraryModule};

pub struct MaskIndicatorCallback {
    pub image: Arc<Image>,
}

impl egui_wgpu::CallbackTrait for MaskIndicatorCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &mut MaskIndicatorRenderResources = resources.get_mut().unwrap();
        resources.prepare(device, queue, self.image.as_ref());
        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        resources: &'a egui_wgpu::CallbackResources,
    ) {
        let resources: &MaskIndicatorRenderResources = resources.get().unwrap();
        resources.paint(render_pass, self.image.as_ref());
    }
}

pub struct MaskIndicatorRenderResources {
    pipeline: wgpu::RenderPipeline,
    bind_group_manager: BindGroupManager,
    bind_group_key_cache: HashMap<u32, BindGroupDescriptorKey>, // image uuid -> key
    texture_sampler: Sampler,
}

impl MaskIndicatorRenderResources {
    pub fn new(runtime: Arc<Runtime>, target_format: wgpu::TextureFormat) -> Self {
        let shader_code = Shader::from_code(include_str!("../shaders/mask_indicator.wgsl"))
            .with_library(ShaderLibraryModule::ColorSpaces)
            .full_code();

        let (pipeline, bind_group_layout) = runtime.create_render_pipeline(
            shader_code.as_str(),
            target_format,
            Some("MaskIndicator"),
        );

        let bind_group_manager = BindGroupManager::new(runtime.clone(), bind_group_layout);

        let texture_sampler = runtime.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        MaskIndicatorRenderResources {
            pipeline,
            bind_group_manager,
            bind_group_key_cache: HashMap::new(),
            texture_sampler,
        }
    }

    pub fn reset(&mut self) {
        self.bind_group_manager.clear_cache();
        self.bind_group_key_cache.clear();
    }

    fn prepare(
        &mut self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        image: &salon_core::runtime::Image,
    ) {
        let bind_group_desc = BindGroupDescriptor {
            entries: vec![
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Texture(image),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.texture_sampler),
                },
            ],
        };
        let bind_group_key = bind_group_desc.to_key();
        self.bind_group_manager.ensure(bind_group_desc);
        self.bind_group_key_cache.insert(image.uuid, bind_group_key);
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
