use std::io::Write;
use std::path::Path;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use sha256::TrySha256Digest;

use crate::runtime::{ColorSpace, Image, ImageReaderJpeg, Toolbox};
use crate::runtime::{ImageFormat, Runtime};
use crate::session::Session;

#[derive(PartialEq, Eq, Hash, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum LibraryImageIdentifier {
    Temp(usize), // images that we no longer have access to after the application closes
    Path(PathBuf),
}

struct LibraryItem {
    image: Option<Arc<Image>>,
    thumbnail: Option<Arc<Image>>,
    thumbnail_path: Option<PathBuf>,
}

pub struct Library {
    items: HashMap<LibraryImageIdentifier, LibraryItem>,
    item_indices: HashMap<LibraryImageIdentifier, usize>,
    items_ordered: Vec<LibraryImageIdentifier>,
    num_temp_images: usize,
    runtime: Arc<Runtime>,
    toolbox: Arc<Toolbox>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct LibraryPersistentState {
    items: Vec<LibraryPersistentStateItem>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct LibraryPersistentStateItem {
    pub path: PathBuf,
    pub thumbnail_path: Option<PathBuf>,
}

impl LibraryPersistentState {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
}

impl Library {
    pub fn new(runtime: Arc<Runtime>, toolbox: Arc<Toolbox>) -> Self {
        Self {
            items: HashMap::new(),
            item_indices: HashMap::new(),
            items_ordered: Vec::new(),
            num_temp_images: 0,
            runtime,
            toolbox,
        }
    }

    pub fn num_images_total(&self) -> usize {
        self.items.len() as usize
    }

    pub fn num_temp_images(&self) -> usize {
        self.num_temp_images
    }

    fn add_item(&mut self, item: LibraryItem, identifier: LibraryImageIdentifier) {
        let old_item = self.items.insert(identifier.clone(), item);
        if old_item.is_none() {
            self.item_indices
                .insert(identifier.clone(), self.items_ordered.len());
            self.items_ordered.push(identifier);
        }
    }

    pub fn add_image_temp(&mut self, image: Arc<Image>) -> LibraryImageIdentifier {
        let temp_image_id = LibraryImageIdentifier::Temp(self.num_temp_images);
        self.num_temp_images += 1;

        let thumbnail = self.compute_thumbnail(image.clone());
        let image = self
            .toolbox
            .convert_image_format(image, ImageFormat::Rgba16Float);
        let image = self
            .toolbox
            .convert_color_space(image, ColorSpace::LinearRGB);
        let library_item = LibraryItem {
            image: Some(image),
            thumbnail: Some(thumbnail),
            thumbnail_path: None,
        };
        self.add_item(library_item, temp_image_id.clone());
        temp_image_id
    }

    pub fn add_item_from_path(&mut self, path: PathBuf) -> LibraryImageIdentifier {
        let id = LibraryImageIdentifier::Path(path);
        let item = LibraryItem {
            image: None,
            thumbnail: None,
            thumbnail_path: None,
        };
        self.add_item(item, id.clone());
        id
    }

    pub fn get_identifier_at_index(&self, index: usize) -> &LibraryImageIdentifier {
        &self.items_ordered[index]
    }

    pub fn get_image_at_index(&mut self, index: usize) -> Option<Arc<Image>> {
        while index < self.items_ordered.len() {
            let identifier = self.items_ordered[index].clone();
            if let Some(image) = self.get_image_from_identifier(&identifier) {
                return Some(image);
            }
            // otherwise this index and the identifier is removed, so try again
        }
        None
    }

    pub fn get_thumbnail_at_index(&mut self, index: usize) -> Option<Arc<Image>> {
        while index < self.items_ordered.len() {
            let identifier = self.items_ordered[index].clone();
            if let Some(thumbnail) = self.get_thumbnail_from_identifier(&identifier) {
                return Some(thumbnail);
            }
            // otherwise this index and the identifier is removed, so try again
        }
        None
    }

    // return the item or delete the identifier
    fn get_fully_loaded_item(
        &mut self,
        identifier: &LibraryImageIdentifier,
    ) -> Option<&LibraryItem> {
        if self.items[identifier].image.is_none() {
            if let LibraryImageIdentifier::Path(ref path) = identifier {
                if let Ok(image) = self.runtime.create_image_from_path(&path) {
                    let image = Arc::new(image);

                    // when loading image from path, always re-compute thumbnail (before format and color space conversions)
                    let thumbnail = self.compute_thumbnail(image.clone());
                    let thumbnail_path = self.save_thumbnail(thumbnail.clone(), path);

                    let image = self
                        .toolbox
                        .convert_image_format(image, ImageFormat::Rgba16Float);
                    let image = self
                        .toolbox
                        .convert_color_space(image, ColorSpace::LinearRGB);
                    {
                        let item = self.items.get_mut(identifier).unwrap();
                        item.image = Some(image);
                        item.thumbnail = Some(thumbnail);
                        if let Some(ref old_thumbnail_path) = item.thumbnail_path {
                            let _ = std::fs::remove_file(old_thumbnail_path);
                        }
                        item.thumbnail_path = thumbnail_path;
                    }
                    return Some(&self.items[identifier]);
                } else {
                    // main image path couldn't be loaded, so delete it and its thumbnail
                    let item = self.items.remove(identifier).unwrap();
                    let index = self.item_indices.remove(identifier).unwrap();
                    self.items_ordered.remove(index);
                    if let Some(ref old_thumbnail_path) = item.thumbnail_path {
                        let _ = std::fs::remove_file(old_thumbnail_path);
                    }
                    return None;
                }
            } else {
                panic!("temp image is empty");
            }
        }

        if self.items[identifier].thumbnail.is_none() {
            let thumbnail =
                self.compute_thumbnail(self.items[identifier].image.as_ref().unwrap().clone());
            self.items.get_mut(identifier).unwrap().thumbnail = Some(thumbnail);
        }

        Some(&self.items[identifier])
    }

    // return the item or delete the identifier
    pub fn get_image_from_identifier(
        &mut self,
        identifier: &LibraryImageIdentifier,
    ) -> Option<Arc<Image>> {
        let item = self.get_fully_loaded_item(identifier)?;
        Some(item.image.as_ref().unwrap().clone())
    }

    // return the item or delete the identifier
    pub fn get_thumbnail_from_identifier(
        &mut self,
        identifier: &LibraryImageIdentifier,
    ) -> Option<Arc<Image>> {
        if let Some(ref thumbnail) = self.items[identifier].thumbnail {
            return Some(thumbnail.clone());
        }
        if let Some(ref thumbnail_path) = self.items[identifier].thumbnail_path {
            if let Ok(thumbnail) = self.runtime.create_image_from_path(thumbnail_path) {
                let thumbnail = Arc::new(thumbnail);
                self.toolbox.generate_mipmap(&thumbnail);
                self.items.get_mut(identifier).unwrap().thumbnail = Some(thumbnail.clone());
                return Some(thumbnail);
            }
        }
        let item = self.get_fully_loaded_item(identifier)?;
        Some(item.thumbnail.as_ref().unwrap().clone())
    }

    pub fn get_persistent_state(&self) -> LibraryPersistentState {
        let mut persistent_items = Vec::new();
        for (identifier, library_item) in self.items.iter() {
            if let LibraryImageIdentifier::Path(ref path) = identifier {
                let item = LibraryPersistentStateItem {
                    path: path.clone(),
                    thumbnail_path: library_item.thumbnail_path.clone(),
                };
                persistent_items.push(item);
            }
        }
        LibraryPersistentState {
            items: persistent_items,
        }
    }

    pub fn load_persistent_state(&mut self, state: LibraryPersistentState) {
        for item in state.items {
            let identifier = self.add_item_from_path(item.path);
            self.items.get_mut(&identifier).unwrap().thumbnail_path = item.thumbnail_path;
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
            self.toolbox.generate_mipmap(&image);
            image
        }
    }

    fn get_thumbnail_path_for_image_path(&self, image_path: &PathBuf) -> Option<PathBuf> {
        if let Ok(digest_str) = image_path.digest() {
            if let Some(storage_dir) = Session::get_persistent_storage_dir() {
                let file_name = digest_str + ".jpg";
                let full_path = storage_dir.join("thumbnails").join(file_name);
                return Some(full_path);
            }
        }
        None
    }

    fn save_thumbnail(
        &self,
        thumbnail: Arc<Image>,
        original_image_path: &PathBuf,
    ) -> Option<PathBuf> {
        if let Some(thumbnail_path) = self.get_thumbnail_path_for_image_path(original_image_path) {
            let mut image_reader = ImageReaderJpeg::new(
                self.runtime.clone(),
                self.toolbox.clone(),
                thumbnail.clone(),
            );
            let result = Some(thumbnail_path.clone());
            std::thread::spawn(move || {
                futures::executor::block_on(async move {
                    std::fs::create_dir_all(thumbnail_path.parent().unwrap())
                        .expect("failed to ensure thumbnail directory");
                    let jpeg_data = image_reader.await_jpeg_data().await;
                    let mut file = std::fs::File::create(&thumbnail_path)
                        .expect("failed to create thumbnail file");
                    file.write_all(&jpeg_data).expect("failed to write file");
                })
            });
            return result;
        }
        None
    }
}
