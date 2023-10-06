use std::{
    collections::HashSet,
    fs::File,
    io::{BufReader, Cursor, Read},
    path::PathBuf,
    sync::Arc,
};

use image::{imageops, DynamicImage, GenericImageView, ImageBuffer, Rgb};
use imagepipe::{ImageSource, Pipeline};

use crate::{
    image::{BitDepth, ColorSpace, Image, ImageProperties},
    utils::mipmap::MipmapGenerator,
};

pub struct Runtime {
    pub adapter: Arc<wgpu::Adapter>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,

    toolbox: Option<ToolBox>,
}

struct ToolBox {
    pub mipmap_generator: MipmapGenerator,
}

impl Runtime {
    pub fn new(
        adapter: Arc<wgpu::Adapter>,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
    ) -> Self {
        let mut runtime = Runtime {
            adapter,
            device,
            queue,
            toolbox: None,
        };
        let toolbox = ToolBox {
            mipmap_generator: MipmapGenerator::new(&runtime),
        };
        runtime.toolbox = Some(toolbox);
        runtime
    }

    pub fn ensure_mipmap(&self, image: &Image) {
        self.toolbox
            .as_ref()
            .unwrap()
            .mipmap_generator
            .generate(self, image);
    }

    pub fn encode_mipmap_generation_command(
        &self,
        image: &Image,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        self.toolbox
            .as_ref()
            .unwrap()
            .mipmap_generator
            .encode_mipmap_generation_command(self, image, encoder);
    }

    pub fn create_compute_pipeline(
        &self,
        wgsl_code: &str,
    ) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(wgsl_code.into()),
            });

        let pipeline = self
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &shader,
                entry_point: "cs_main",
            });

        let bind_group_layout = pipeline.get_bind_group_layout(0);
        (pipeline, bind_group_layout)
    }

    pub fn create_render_pipeline(
        &self,
        wgsl_code: &str,
        target_format: wgpu::TextureFormat,
    ) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(wgsl_code.into()),
            });

        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: None,
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(target_format.into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

        let bind_group_layout = pipeline.get_bind_group_layout(0);
        (pipeline, bind_group_layout)
    }

    pub fn create_image_of_properties(&self, properties: ImageProperties) -> Image {
        let size = wgpu::Extent3d {
            width: properties.dimensions.0,
            height: properties.dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: size,
            mip_level_count: Image::mip_level_count(&properties.dimensions),
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: properties.to_wgpu_texture_format(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_view_base_mip = texture.create_view(&wgpu::TextureViewDescriptor {
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });
        Image {
            properties,
            texture,
            texture_view,
            texture_view_base_mip,
            uuid: crate::utils::uuid::get_next_uuid(),
        }
    }

    pub fn create_image_from_path(&self, path: &PathBuf) -> Result<Image, String> {
        let Ok(image_bytes) = std::fs::read(&path) else {
            return Err("could not find file at path ".to_string() + path.to_str().unwrap_or(""));
        };
        let Some(ext) = path.extension() else {
            return Err("missing file extension".to_owned());
        };
        self.create_image_from_bytes_and_extension(image_bytes.as_slice(), ext.to_str().unwrap())
    }

    pub fn create_image_from_bytes_and_extension(
        &self,
        image_bytes: &[u8],
        extension: &str,
    ) -> Result<Image, String> {
        let extension = extension.to_lowercase();
        if extension == "jpg" || extension == "jpeg" || extension == "png" {
            return self.create_image_from_bytes_jpg_png(image_bytes);
        }
        let raw_extensions = HashSet::from([
            "raf", // fujifilm
            "crw", "cr2", // canon
            "nrw", "nef", // nikon
            "arw", "srf", "sr2", // sony,
            "rw2", // Panasonic, Leica,
            "3fr", // Hasselblad
        ]);
        if raw_extensions.contains(extension.as_str()) {
            return self.create_image_from_bytes_raw(image_bytes);
        }

        Err("unsupported image format: ".to_owned() + extension.as_str())
    }

    pub fn create_image_from_bytes_raw(&self, image_bytes: &[u8]) -> Result<Image, String> {
        let decode_result = rawloader::decode(&mut Cursor::new(image_bytes));
        let Ok(raw) = decode_result else {
            return Err(decode_result.err().unwrap().to_string());
        };

        let source = ImageSource::Raw(raw);
        let Ok(mut pipeline) = Pipeline::new_from_source(source) else {
            return Err("imagepipe cannot decode file".to_owned());
        };

        pipeline.run(None);
        let Ok(image) = pipeline.output_16bit(None) else {
            return Err("imagepipe cannot output file".to_owned());
        };

        let image = ImageBuffer::<Rgb<u16>, Vec<u16>>::from_raw(
            image.width as u32,
            image.height as u32,
            image.data,
        );

        let image = image::DynamicImage::ImageRgb16(image.expect("cannot create DynamicImage"));
        Ok(self.create_image_from_dynamic_image(image))
    }

    pub fn create_image_from_bytes_jpg_png(&self, image_bytes: &[u8]) -> Result<Image, String> {
        let Ok(mut img) = image::load_from_memory(image_bytes) else {
            return Err("image::load_from_memory failed".to_owned());
        };

        // use exif to fix image orientation
        // https://github.com/image-rs/image/issues/1958
        let exif_reader = exif::Reader::new();
        let mut cursor = Cursor::new(image_bytes);
        let exif = exif_reader.read_from_container(&mut cursor);

        let orientation: u32 = match exif {
            Ok(exif) => match exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY) {
                Some(orientation) => match orientation.value.get_uint(0) {
                    Some(v @ 1..=8) => v,
                    _ => 1,
                },
                None => 1,
            },
            Err(_) => 1,
        };

        if orientation == 2 {
            img = DynamicImage::ImageRgba8(imageops::flip_horizontal(&img));
        } else if orientation == 3 {
            img = DynamicImage::ImageRgba8(imageops::rotate180(&img));
        } else if orientation == 4 {
            img = DynamicImage::ImageRgba8(imageops::flip_horizontal(&img));
        } else if orientation == 5 {
            img = DynamicImage::ImageRgba8(imageops::rotate90(&img));
            img = DynamicImage::ImageRgba8(imageops::flip_horizontal(&img));
        } else if orientation == 6 {
            img = DynamicImage::ImageRgba8(imageops::rotate90(&img));
        } else if orientation == 7 {
            img = DynamicImage::ImageRgba8(imageops::rotate270(&img));
            img = DynamicImage::ImageRgba8(imageops::flip_horizontal(&img));
        } else if orientation == 8 {
            img = DynamicImage::ImageRgba8(imageops::rotate270(&img));
        }
        Ok(self.create_image_from_dynamic_image(img))
    }

    pub fn create_image_from_dynamic_image(&self, dynamic_image: image::DynamicImage) -> Image {
        let dimensions = dynamic_image.dimensions();
        let properties = ImageProperties {
            dimensions,
            bit_depth: BitDepth::Depth8,
            color_space: ColorSpace::sRGB,
        };
        let result = self.create_image_of_properties(properties);
        let rgba = dynamic_image.to_rgba8();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        self.queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &result.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );
        self.ensure_mipmap(&result);
        result
    }
}
