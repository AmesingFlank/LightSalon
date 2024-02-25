use std::path::Path;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::runtime::{ColorSpace, Image, Toolbox};
use crate::runtime::{ImageFormat, Runtime};

#[derive(PartialEq, Eq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub enum LibraryImageIdentifier {
    Temp(usize), // images that we no longer have access to after the application closes
    Path(PathBuf),
}

pub struct Library {
    images: HashMap<LibraryImageIdentifier, Arc<Image>>,
    images_order: Vec<LibraryImageIdentifier>,
    num_temp_images: usize,
    runtime: Arc<Runtime>,
    toolbox: Arc<Toolbox>,
    persistent_state: LibraryPersistentState,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct LibraryPersistentState {
    pub paths: Vec<PathBuf>,
}

impl LibraryPersistentState {
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
        }
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
            persistent_state: LibraryPersistentState::new(),
        }
    }

    pub fn num_images_total(&self) -> usize {
        self.images.len() as usize
    }

    pub fn num_temp_images(&self) -> usize {
        self.num_temp_images
    }

    pub fn add_image(&mut self, image: Arc<Image>, identifier: LibraryImageIdentifier) {
        let image = self
            .toolbox
            .convert_image_format(image, ImageFormat::Rgba16Float);
        let image = self
            .toolbox
            .convert_color_space(image, ColorSpace::LinearRGB);
        let old_image = self.images.insert(identifier.clone(), image);
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
        self.images[identifier].clone()
    }

    pub fn get_thumbnail_at_index(&mut self, index: usize) -> Arc<Image> {
        let identifier = &self.images_order[index];
        self.images[identifier].clone()
    }

    pub fn get_image_from_identifier(&mut self, identifier: &LibraryImageIdentifier) -> Arc<Image> {
        self.images[&identifier].clone()
    }

    pub fn get_thumbnail_from_identifier(
        &mut self,
        identifier: &LibraryImageIdentifier,
    ) -> Arc<Image> {
        self.images[&identifier].clone()
    }

    pub fn get_persistent_state(&self) -> &LibraryPersistentState {
        &self.persistent_state
    }
}
