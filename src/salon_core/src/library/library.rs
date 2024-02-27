use std::path::Path;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::runtime::{ColorSpace, Image, Toolbox};
use crate::runtime::{ImageFormat, Runtime};

#[derive(PartialEq, Eq, Hash, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum LibraryImageIdentifier {
    Temp(usize), // images that we no longer have access to after the application closes
    Path(PathBuf),
}

pub struct LibraryImage {
    image: Option<Arc<Image>>,
    thumbnail: Option<Arc<Image>>,
}

pub struct Library {
    images: HashMap<LibraryImageIdentifier, LibraryImage>,
    images_order: Vec<LibraryImageIdentifier>,
    num_temp_images: usize,
    runtime: Arc<Runtime>,
    toolbox: Arc<Toolbox>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct LibraryPersistentState {
    pub paths: Vec<PathBuf>,
}

impl LibraryPersistentState {
    pub fn new() -> Self {
        Self { paths: Vec::new() }
    }
}

impl Library {
    pub fn new(runtime: Arc<Runtime>, toolbox: Arc<Toolbox>) -> Self {
        Self {
            images: HashMap::new(),
            images_order: Vec::new(),
            num_temp_images: 0,
            runtime,
            toolbox,
        }
    }

    pub fn num_images_total(&self) -> usize {
        self.images.len() as usize
    }

    pub fn num_temp_images(&self) -> usize {
        self.num_temp_images
    }

    pub fn add_image(&mut self, image: Arc<Image>, identifier: LibraryImageIdentifier) {
        let thumbnail = self.compute_thumbnail(image.clone());
        let image = self
            .toolbox
            .convert_image_format(image, ImageFormat::Rgba16Float);
        let image = self
            .toolbox
            .convert_color_space(image, ColorSpace::LinearRGB);
        let library_image = LibraryImage {
            image: Some(image),
            thumbnail: Some(thumbnail),
        };
        let old_image = self.images.insert(identifier.clone(), library_image);
        if old_image.is_none() {
            self.images_order.push(identifier);
        }
    }

    pub fn add_image_temp(&mut self, image: Arc<Image>) -> LibraryImageIdentifier {
        let temp_image_id = LibraryImageIdentifier::Temp(self.num_temp_images);
        self.num_temp_images += 1;
        self.add_image(image, temp_image_id.clone());
        temp_image_id
    }

    pub fn add_image_from_path(&mut self, path: PathBuf) -> Result<LibraryImageIdentifier, String> {
        let image = self.runtime.create_image_from_path(&path)?;
        let id = LibraryImageIdentifier::Path(path);
        self.add_image(Arc::new(image), id.clone());
        Ok(id)
    }

    pub fn get_identifier_at_index(&self, index: usize) -> &LibraryImageIdentifier {
        &self.images_order[index]
    }

    pub fn get_image_at_index(&mut self, index: usize) -> Arc<Image> {
        let identifier = &self.images_order[index];
        self.get_image_from_identifier(&identifier.clone())
    }

    pub fn get_thumbnail_at_index(&mut self, index: usize) -> Arc<Image> {
        let identifier = &self.images_order[index];
        self.get_thumbnail_from_identifier(&identifier.clone())
    }

    fn ensure_loaded(&mut self, identifier: &LibraryImageIdentifier) -> &LibraryImage {
        if self.images[identifier].image.is_none() {
            if let LibraryImageIdentifier::Path(ref path) = identifier {
                let image = self
                    .runtime
                    .create_image_from_path(&path)
                    .expect("failed to create image from path");
                let image = Arc::new(image);
                self.images.get_mut(identifier).unwrap().image = Some(image);
            } else {
                panic!("cannot load from a non-path identifier {:?}", identifier);
            }
        }

        if self.images[identifier].thumbnail.is_none() {
            let thumbnail =
                self.compute_thumbnail(self.images[identifier].image.as_ref().unwrap().clone());
            self.images.get_mut(identifier).unwrap().thumbnail = Some(thumbnail)
        }
        &self.images[identifier]
    }

    pub fn get_image_from_identifier(&mut self, identifier: &LibraryImageIdentifier) -> Arc<Image> {
        self.ensure_loaded(identifier);
        self.images[identifier].image.as_ref().unwrap().clone()
    }

    pub fn get_thumbnail_from_identifier(
        &mut self,
        identifier: &LibraryImageIdentifier,
    ) -> Arc<Image> {
        self.ensure_loaded(identifier);
        self.images[identifier].thumbnail.as_ref().unwrap().clone()
    }

    pub fn get_persistent_state(&self) -> LibraryPersistentState {
        let mut paths = Vec::new();
        for pair in self.images.iter() {
            if let LibraryImageIdentifier::Path(ref path) = pair.0 {
                paths.push(path.clone())
            }
        }
        LibraryPersistentState { paths }
    }

    pub fn load_persistent_state(&mut self, state: LibraryPersistentState) {
        for path in state.paths {
            let _ = self.add_image_from_path(path);
        }
    }

    fn compute_thumbnail(&self, image: Arc<Image>) -> Arc<Image> {
        let thumbnail_min_dimension_size = 400.0 as f32;
        let factor = thumbnail_min_dimension_size
            / (image.properties.dimensions.0).min(image.properties.dimensions.1) as f32;
        if factor < 0.5 {
            self.toolbox.generate_mipmap(&image);
            let thumbnail = self.toolbox.resize_image(image, factor);
            self.toolbox.generate_mipmap(&thumbnail);
            thumbnail
        } else {
            image
        }
    }
}
