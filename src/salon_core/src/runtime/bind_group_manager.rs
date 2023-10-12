use std::sync::Arc;

use crate::{buffer::Buffer, image::Image};

use super::Runtime;

pub struct BindGroupManager {
    layout: wgpu::BindGroupLayout,
    runtime: Arc<Runtime>,
}

impl BindGroupManager {
    pub fn new(layout: wgpu::BindGroupLayout, runtime: Arc<Runtime>) -> Self {
        Self { layout, runtime }
    }
    fn make_bind_group<'a>(&'a self, descriptor: &'a BindGroupDescriptor<'a>) -> wgpu::BindGroup {
        let mut entries_wgpu = Vec::new();
        for e in descriptor.entries.iter() {
            entries_wgpu.push(e.to_wgpu())
        }
        self.runtime
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.layout,
                entries: entries_wgpu.as_slice(),
            })
    }
}

pub struct BindGroupDescriptor<'a> {
    pub entries: Vec<BindGroupEntry<'a>>,
}

impl<'a> BindGroupDescriptor<'a> {}

pub struct BindGroupEntry<'a> {
    pub binding: u32,
    pub resource: BindingResource<'a>,
}

impl<'a> BindGroupEntry<'a> {
    fn to_wgpu(&'a self) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding: self.binding,
            resource: self.resource.to_wgpu(),
        }
    }
}

pub enum BindingResource<'a> {
    Buffer(&'a Buffer),
    Image(&'a Image),
    ImageSingleMip(&'a Image, u32),
}

impl<'a> BindingResource<'a> {
    fn to_wgpu(&'a self) -> wgpu::BindingResource<'a> {
        match *self {
            BindingResource::Buffer(buffer) => buffer.buffer.as_entire_binding(),
            BindingResource::Image(img) => wgpu::BindingResource::TextureView(&img.texture_view),
            BindingResource::ImageSingleMip(img, ref mip) => {
                wgpu::BindingResource::TextureView(&img.texture_view_single_mip[*mip as usize])
            }
        }
    }
}

struct BindGroupDescriptorKey {
    pub entries: Vec<BindGroupEntryKey>,
}

struct BindGroupEntryKey {
    pub binding: u32,
    pub resource: BindingResourceKey,
}

enum BindingResourceKey {
    Buffer(u32),
    Image(u32),
    ImageSingleMip(u32, u32),
}
