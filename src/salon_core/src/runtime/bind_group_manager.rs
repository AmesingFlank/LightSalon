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
    pub entries: &'a [BindGroupEntry<'a>],
}

impl<'a> BindGroupDescriptor<'a> {
    fn to_key(&self) -> BindGroupDescriptorKey {
        let mut entries = Vec::new();
        for e in self.entries.iter() {
            entries.push(e.to_key())
        }
        BindGroupDescriptorKey { entries }
    }
}

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
    fn to_key(&self) -> BindGroupEntryKey {
        BindGroupEntryKey {
            binding: self.binding,
            resource: self.resource.to_key(),
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

    fn to_key(&self) -> BindingResourceKey {
        match *self {
            BindingResource::Buffer(buffer) => BindingResourceKey::Buffer(buffer.uuid),
            BindingResource::Image(img) => BindingResourceKey::Image(img.uuid),
            BindingResource::ImageSingleMip(img, ref mip) => {
                BindingResourceKey::ImageSingleMip(img.uuid, *mip)
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
