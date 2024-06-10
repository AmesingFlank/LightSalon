use std::{collections::HashMap, hash::Hash, sync::Arc};

use crate::runtime::{Buffer, Image, Runtime, Sampler};

pub struct BindGroupManager {
    layout: wgpu::BindGroupLayout,
    runtime: Arc<Runtime>,
    cache: HashMap<BindGroupDescriptorKey, wgpu::BindGroup>,
    label: Option<String>,
}

impl BindGroupManager {
    pub fn new(runtime: Arc<Runtime>, layout: wgpu::BindGroupLayout) -> Self {
        Self {
            layout,
            runtime,
            cache: HashMap::new(),
            label: None,
        }
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    pub fn get_or_create<'a>(
        &'a mut self,
        descriptor: BindGroupDescriptor<'a>,
    ) -> &'a wgpu::BindGroup {
        let layout = &self.layout;
        let runtime = self.runtime.as_ref();
        let key = descriptor.to_key();
        let label = match self.label.as_ref() {
            Some(l) => Some(l.as_str()),
            None => None,
        };

        assert!(
            self.cache.len() < 100,
            "BindGroupManager cache size over 100! Please clear the cache regularly. BindGroups kept in memory may stop wgpu from freeing GPU memory of textures/buffers that are destroyed."
        );

        self.cache
            .entry(key)
            .or_insert_with(|| descriptor.make_bind_group(runtime, layout, label))
    }

    pub fn ensure<'a>(&'a mut self, descriptor: BindGroupDescriptor<'a>) {
        self.get_or_create(descriptor);
    }

    pub fn get_from_key_or_panic<'a>(
        &'a self,
        key: &BindGroupDescriptorKey,
    ) -> &'a wgpu::BindGroup {
        self.cache
            .get(key)
            .expect("A bind group corresponding to this descriptor does not exist")
    }

    pub fn clear_cache(&mut self) {
        // In wgpu, BindGroups stored in memory may stop textures/buffer GPU memory from being freed,
        // so we need to clear the cache regularly to prevent memory leaks.
        self.cache.clear();
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
        label: Option<&'a str>,
    ) -> wgpu::BindGroup {
        let mut entries_wgpu = Vec::new();
        for e in self.entries.iter() {
            entries_wgpu.push(e.to_wgpu())
        }
        runtime
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label,
                layout: layout,
                entries: entries_wgpu.as_slice(),
            })
    }

    pub fn to_key(&self) -> BindGroupDescriptorKey {
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
    TextureSingleMip(&'a Image, u32),
    TextureStorage(&'a Image, u32),
    Sampler(&'a Sampler),
}

impl<'a> BindingResource<'a> {
    fn to_wgpu(&'a self) -> wgpu::BindingResource<'a> {
        match *self {
            BindingResource::Buffer(buffer) => buffer.buffer.as_entire_binding(),
            BindingResource::Texture(img) => wgpu::BindingResource::TextureView(&img.texture_view),
            BindingResource::TextureSingleMip(img, ref mip) => {
                wgpu::BindingResource::TextureView(&img.texture_view_single_mip[*mip as usize])
            }
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
            BindingResource::TextureSingleMip(img, ref mip) => {
                BindingResourceKey::TextureSingleMip(img.uuid, *mip)
            }
            BindingResource::TextureStorage(img, ref mip) => {
                BindingResourceKey::TextureStorage(img.uuid, *mip)
            }
            BindingResource::Sampler(s) => BindingResourceKey::Sampler(s.uuid),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct BindGroupDescriptorKey {
    pub entries: Vec<BindGroupEntryKey>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct BindGroupEntryKey {
    pub binding: u32,
    pub resource: BindingResourceKey,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum BindingResourceKey {
    Buffer(u32),
    Texture(u32),
    TextureSingleMip(u32, u32),
    TextureStorage(u32, u32),
    Sampler(u32),
}
