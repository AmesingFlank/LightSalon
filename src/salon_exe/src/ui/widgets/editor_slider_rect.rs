use std::mem::size_of;
use std::sync::Arc;
use std::{collections::HashMap};

use eframe::egui_wgpu::ScreenDescriptor;
use eframe::{egui, egui_wgpu};

use salon_core::runtime::{
    BindGroupDescriptor, BindGroupDescriptorKey, BindGroupEntry, BindGroupManager, BindingResource,
    Runtime,
};
use salon_core::runtime::{Buffer, BufferProperties, RingBuffer};
use salon_core::runtime::{ColorSpace};
use salon_core::shader::{Shader, ShaderLibraryModule};


pub struct EditorSliderRectCallback {
    pub left_color: [f32; 3],
    pub right_color: [f32; 3],
    pub color_space: ColorSpace,
    pub rect_id: egui::Id,
}

impl egui_wgpu::CallbackTrait for EditorSliderRectCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &mut EditorSliderRectRenderResources = resources.get_mut().unwrap();
        resources.prepare(
            device,
            queue,
            self.left_color,
            self.right_color,
            self.color_space,
            self.rect_id,
        );
        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        resources: &'a egui_wgpu::CallbackResources,
    ) {
        let resources: &EditorSliderRectRenderResources = resources.get().unwrap();
        resources.paint(render_pass, self.rect_id);
    }
}

pub struct EditorSliderRectRenderResources {
    pipeline: wgpu::RenderPipeline,
    bind_group_manager: BindGroupManager,
    bind_group_key_cache: HashMap<egui::Id, BindGroupDescriptorKey>, // ring buffer uuid -> key
    ring_buffer: RingBuffer,
}

impl EditorSliderRectRenderResources {
    pub fn new(runtime: Arc<Runtime>, target_format: wgpu::TextureFormat) -> Self {
        let shader_code = Shader::from_code(include_str!("../shaders/editor_slider_rect.wgsl"))
            .with_library(ShaderLibraryModule::ColorSpaces)
            .full_code();

        let (pipeline, bind_group_layout) = runtime.create_render_pipeline(
            shader_code.as_str(),
            target_format,
            Some("EditorSliderRect"),
        );

        let bind_group_manager = BindGroupManager::new(runtime.clone(), bind_group_layout);

        let ring_buffer = RingBuffer::new(
            runtime.clone(),
            BufferProperties {
                size: size_of::<f32>() * (12),
                host_readable: false,
            },
        );

        EditorSliderRectRenderResources {
            pipeline,
            bind_group_manager,
            bind_group_key_cache: HashMap::new(),
            ring_buffer,
        }
    }

    pub fn reset(&mut self) {
        self.ring_buffer.mark_all_available();
        self.bind_group_manager.clear_cache();
        self.bind_group_key_cache.clear();
    }

    fn prepare(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        left_color: [f32; 3],
        right_color: [f32; 3],
        color_space: ColorSpace,
        rect_id: egui::Id,
    ) {
        let buffer: &Buffer = self.ring_buffer.get();
        queue.write_buffer(
            &buffer.buffer,
            0,
            bytemuck::cast_slice(&[
                left_color[0],
                left_color[1],
                left_color[2],
                1.0,
                right_color[0],
                right_color[1],
                right_color[2],
                1.0,
                color_space as u32 as f32,
            ]),
        );

        let bind_group_desc = BindGroupDescriptor {
            entries: vec![BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(buffer),
            }],
        };
        let bind_group_key = bind_group_desc.to_key();
        self.bind_group_manager.ensure(bind_group_desc);
        self.bind_group_key_cache.insert(rect_id, bind_group_key);
    }

    fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>, rect_id: egui::Id) {
        let bind_group_key = self.bind_group_key_cache.get(&rect_id).unwrap();
        let bind_group = self
            .bind_group_manager
            .get_from_key_or_panic(bind_group_key);

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}
