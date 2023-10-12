use std::{collections::HashMap, hash::Hash, sync::Arc};

use crate::{buffer::Buffer, image::Image, sampler::Sampler};

use super::{runtime, Runtime};

pub struct BindGroupManager {
    layout: wgpu::BindGroupLayout,
    runtime: Arc<Runtime>,
    cache: HashMap<BindGroupDescriptorKey, wgpu::BindGroup>,
}

impl BindGroupManager {
    pub fn new(runtime: Arc<Runtime>, layout: wgpu::BindGroupLayout) -> Self {
        Self {
            layout,
            runtime,
            cache: HashMap::new(),
        }
    }
    pub fn get<'a>(&'a mut self, descriptor: BindGroupDescriptor<'a>) -> &'a wgpu::BindGroup {
        let layout = &self.layout;
        let runtime = self.runtime.as_ref();
        let key = descriptor.to_key();

        self.cache
            .entry(key)
            .or_insert_with(|| descriptor.make_bind_group(runtime, layout))
    }
}

pub struct BindGroupDescriptor<'a> {
    pub entries: Vec<BindGroupEntry<'a>>,
}

impl<'a> BindGroupDescriptor<'a> {
    fn make_bind_group(
        &'a self,
        runtime: &'a Runtime,
        layout: &'a wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        let mut entries_wgpu = Vec::new();
        for e in self.entries.iter() {
            entries_wgpu.push(e.to_wgpu())
        }
        runtime
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: layout,
                entries: entries_wgpu.as_slice(),
            })
    }

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
    Texture(&'a Image),
    TextureStorage(&'a Image, u32),
    Sampler(&'a Sampler),
}

impl<'a> BindingResource<'a> {
    fn to_wgpu(&'a self) -> wgpu::BindingResource<'a> {
        match *self {
            BindingResource::Buffer(buffer) => buffer.buffer.as_entire_binding(),
            BindingResource::Texture(img) => wgpu::BindingResource::TextureView(&img.texture_view),
            BindingResource::TextureStorage(img, ref mip) => {
                wgpu::BindingResource::TextureView(&img.texture_view_single_mip[*mip as usize])
            }
            BindingResource::Sampler(s) => wgpu::BindingResource::Sampler(&s.sampler),
        }
    }

    fn to_key(&self) -> BindingResourceKey {
        match *self {
            BindingResource::Buffer(buffer) => BindingResourceKey::Buffer(buffer.uuid),
            BindingResource::Texture(img) => BindingResourceKey::Texture(img.uuid),
            BindingResource::TextureStorage(img, ref mip) => {
                BindingResourceKey::TextureStorage(img.uuid, *mip)
            }
            BindingResource::Sampler(s) => BindingResourceKey::Sampler(s.uuid),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct BindGroupDescriptorKey {
    pub entries: Vec<BindGroupEntryKey>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct BindGroupEntryKey {
    pub binding: u32,
    pub resource: BindingResourceKey,
}

#[derive(PartialEq, Eq, Hash, Clone)]
enum BindingResourceKey {
    Buffer(u32),
    Texture(u32),
    TextureStorage(u32, u32),
    Sampler(u32),
}
