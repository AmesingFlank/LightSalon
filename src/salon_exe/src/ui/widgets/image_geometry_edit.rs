use std::collections::HashMap;
use std::mem::size_of;
use std::sync::Arc;

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
use salon_core::utils::rectangle::Rectangle;

use crate::ui::utils::get_max_image_size;

pub struct ImageGeometryEditCallback {
    pub full_image: Arc<Image>,
    pub rotation_degrees: f32,
    pub crop_rect: Rectangle,
    pub ui_max_rect: egui::Rect,
}

impl ImageGeometryEditCallback {
    pub fn required_allocated_rect(&self) -> egui::Rect {
        self.ui_max_rect
    }

    fn full_image_size(&self) -> egui::Vec2 {
        let full_image_ui_rect_size = get_max_image_size(
            self.full_image.aspect_ratio(),
            self.ui_max_rect.width(),
            self.ui_max_rect.height(),
        );
        full_image_ui_rect_size * 0.9
    }

    pub fn cropped_image_ui_rect(&self) -> egui::Rect {
        let full_image_size = self.full_image_size();
        let cropped_image_size = full_image_size
            * egui::Vec2 {
                x: self.crop_rect.size.x,
                y: self.crop_rect.size.y,
            };
        egui::Rect::from_center_size(self.ui_max_rect.center(), cropped_image_size)
    }
}

impl egui_wgpu::CallbackTrait for ImageGeometryEditCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &mut ImageGeometryEditRenderResources = resources.get_mut().unwrap();
        resources.prepare(device, queue, self);
        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        resources: &'a egui_wgpu::CallbackResources,
    ) {
        let resources: &ImageGeometryEditRenderResources = resources.get().unwrap();
        resources.paint(render_pass, &self.full_image.as_ref());
    }
}

pub struct ImageGeometryEditRenderResources {
    pipeline: wgpu::RenderPipeline,
    bind_group_manager: BindGroupManager,
    bind_group_key_cache: HashMap<u32, BindGroupDescriptorKey>, // image uuid -> key
    ring_buffer: RingBuffer,
    texture_sampler: Sampler,
}

impl ImageGeometryEditRenderResources {
    pub fn new(runtime: Arc<Runtime>, target_format: wgpu::TextureFormat) -> Self {
        let shader_code = Shader::from_code(include_str!("../shaders/image_geometry_edit.wgsl"))
            .with_library(ShaderLibraryModule::ColorSpaces)
            .full_code();

        let (pipeline, bind_group_layout) = runtime.create_render_pipeline(
            shader_code.as_str(),
            target_format,
            Some("ImageGeometryEdit"),
        );

        let bind_group_manager = BindGroupManager::new(runtime.clone(), bind_group_layout)
            .with_label("ImageGeometryEdit".to_owned());

        let ring_buffer = RingBuffer::new(
            runtime.clone(),
            BufferProperties {
                size: size_of::<u32>() * 1 + 8 * size_of::<f32>(),
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

        ImageGeometryEditRenderResources {
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
        render_call: &ImageGeometryEditCallback,
    ) {
        let buffer = self.ring_buffer.get();
        queue.write_buffer(
            &buffer.buffer,
            0,
            bytemuck::cast_slice(&[render_call.full_image.properties.color_space as u32]),
        );
        let rotation_degrees = render_call.rotation_degrees;
        let rotation_radians = rotation_degrees.to_radians();

        let full_image_ui_size = render_call.full_image_size();
        let allocated_rect = render_call.required_allocated_rect();
        let width = 2.0 * full_image_ui_size.x / allocated_rect.width();
        let height = 2.0 * full_image_ui_size.y / allocated_rect.height();

        let center_x = width * (render_call.crop_rect.center.x - 0.5) * -1.0;
        let center_y = height * (render_call.crop_rect.center.y - 0.5) * -1.0 * -1.0;

        let crop_rect_width = width * render_call.crop_rect.size.x;
        let crop_rect_height = height * render_call.crop_rect.size.y;

        let render_target_aspect_ratio = allocated_rect.width() / allocated_rect.height();
        queue.write_buffer(
            &buffer.buffer,
            size_of::<u32>() as u64,
            bytemuck::cast_slice(&[
                rotation_radians,
                center_x,
                center_y,
                width,
                height,
                crop_rect_width,
                crop_rect_height,
                render_target_aspect_ratio,
            ]),
        );

        let bind_group_desc = BindGroupDescriptor {
            entries: vec![
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(buffer),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Texture(&render_call.full_image),
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
            .insert(render_call.full_image.uuid, bind_group_key);
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
