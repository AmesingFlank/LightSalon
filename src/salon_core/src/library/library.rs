use std::collections::HashSet;
use std::io::Write;
use std::path::Path;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use sha256::TrySha256Digest;

use crate::runtime::{ColorSpace, Image, ImageReaderJpeg, Toolbox};
use crate::runtime::{ImageFormat, Runtime};
use crate::session::Session;
use crate::utils::uuid::{get_next_uuid, Uuid};

use super::{album, Album, AlbumPersistentState};

#[derive(PartialEq, Eq, Hash, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum LibraryImageIdentifier {
    Temp(Uuid), // images that we no longer have access to after the application closes
    Path(PathBuf),
}

struct LibraryItem {
    image: Option<Arc<Image>>,
    thumbnail: Option<Arc<Image>>,
    thumbnail_path: Option<PathBuf>,
    albums: HashSet<usize>,
}

pub struct Library {
    items: HashMap<LibraryImageIdentifier, LibraryItem>,
    item_indices: HashMap<LibraryImageIdentifier, usize>,
    items_ordered: Vec<LibraryImageIdentifier>,
    runtime: Arc<Runtime>,
    toolbox: Arc<Toolbox>,
    albums: Vec<Album>,

    // items that cannot be found
    unavailable_items: HashSet<LibraryImageIdentifier>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct LibraryPersistentState {
    items: Vec<LibraryPersistentStateItem>,
    albums: Vec<AlbumPersistentState>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct LibraryPersistentStateItem {
    pub path: PathBuf,
    pub thumbnail_path: Option<PathBuf>,
}

impl LibraryPersistentState {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            albums: Vec::new(),
        }
    }
}

impl Library {
    pub fn new(runtime: Arc<Runtime>, toolbox: Arc<Toolbox>) -> Self {
        Self {
            items: HashMap::new(),
            item_indices: HashMap::new(),
            items_ordered: Vec::new(),
            albums: Vec::new(),
            runtime,
            toolbox,
            unavailable_items: HashSet::new(),
        }
    }

    pub fn num_images_total(&self) -> usize {
        self.items.len() as usize
    }

    fn add_item(&mut self, item: LibraryItem, identifier: LibraryImageIdentifier) {
        let old_item = self.items.insert(identifier.clone(), item);
        if old_item.is_none() {
            self.item_indices
                .insert(identifier.clone(), self.items_ordered.len());
            self.items_ordered.push(identifier);
        }
    }

    pub fn add_image_temp(
        &mut self,
        image: Arc<Image>,
        album: Option<usize>,
    ) -> LibraryImageIdentifier {
        let temp_image_id = LibraryImageIdentifier::Temp(get_next_uuid());

        let thumbnail = self.compute_thumbnail(image.clone());
        let image = self
            .toolbox
            .convert_image_format(image, ImageFormat::Rgba16Float);
        let image = self
            .toolbox
            .convert_color_space(image, ColorSpace::LinearRGB);
        let mut library_item = LibraryItem {
            image: Some(image),
            thumbnail: Some(thumbnail),
            thumbnail_path: None,
            albums: HashSet::new(),
        };
        if let Some(album) = album {
            library_item.albums.insert(album);
        }
        self.add_item(library_item, temp_image_id.clone());
        temp_image_id
    }

    pub fn add_item_from_path(
        &mut self,
        path: PathBuf,
        album: Option<usize>,
    ) -> LibraryImageIdentifier {
        let id = LibraryImageIdentifier::Path(path);
        let mut item = LibraryItem {
            image: None,
            thumbnail: None,
            thumbnail_path: None,
            albums: HashSet::new(),
        };
        if let Some(album) = album {
            item.albums.insert(album);
        }
        self.add_item(item, id.clone());
        id
    }

    pub fn add_album_from_directory(&mut self, dir_path: PathBuf) -> usize {
        for i in 0..self.albums.len() {
            if self.albums[i].directory == Some(dir_path.clone()) {
                self.enumerate_album_images(i);
                return i;
            }
        }
        let mut name = "New Album".to_owned();
        if let Some(directory_name) = dir_path.file_name() {
            if let Some(directory_name_str) = directory_name.to_str() {
                name = directory_name_str.to_owned();
            }
        }
        let album = Album::new(name, Some(dir_path), Vec::new());
        let album_index = self.albums.len();
        self.albums.push(album);
        self.enumerate_album_images(album_index);
        album_index
    }

    fn enumerate_album_images(&mut self, album_index: usize) {
        let mut all_images = self.albums[album_index].additional_images.clone();
        {
            let album = &mut self.albums[album_index];

            if let Some(ref path) = album.directory {
                let mut images_in_dir = Vec::new();
                Self::enumerate_images_in_directory(path, &mut images_in_dir);
                for path in images_in_dir {
                    all_images.push(LibraryImageIdentifier::Path(path));
                }
            }
            album.all_images_ordered = all_images.clone();
            album.all_images_indices.clear();
            for i in 0..album.all_images_ordered.len() {
                album
                    .all_images_indices
                    .insert(album.all_images_ordered[i].clone(), i);
            }
        }
        for identifier in all_images.iter() {
            if let Some(item) = self.items.get_mut(identifier) {
                item.albums.insert(album_index);
            } else {
                match identifier {
                    LibraryImageIdentifier::Path(path) => {
                        self.add_item_from_path(path.clone(), Some(album_index));
                    }
                    _ => {
                        panic!("expecting the identifier to be a LibraryImageIdentifier::Path")
                    }
                }
            }
        }
    }

    fn enumerate_images_in_directory(dir: &PathBuf, images: &mut Vec<PathBuf>) {
        if dir.is_dir() {
            if let Ok(read) = std::fs::read_dir(dir) {
                for entry in read {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(ext) = path.extension() {
                                if let Some(ext) = ext.to_str() {
                                    let ext = ext.to_lowercase();
                                    if ext == "jpg" || ext == "jpeg" || ext == "png" {
                                        images.push(path);
                                    }
                                }
                            }
                        } else if path.is_dir() {
                            Self::enumerate_images_in_directory(&path, images);
                        }
                    }
                }
            }
        }
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
                    self.unavailable_items.remove(identifier);

                    return Some(&self.items[identifier]);
                } else {
                    self.unavailable_items.insert(identifier.clone());
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
        if !self.items.contains_key(identifier) {
            // the identifier could have been removed
            return None;
        }
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

    pub fn get_persistent_state(&mut self) -> LibraryPersistentState {
        // these items are found to be unavailable, so remove them from the library
        let mut unavailable_item_indices = Vec::new();
        for unavailable_item_identifier in self.unavailable_items.iter() {
            let item = self.items.remove(unavailable_item_identifier).unwrap();
            let index = self
                .item_indices
                .remove(unavailable_item_identifier)
                .unwrap();
            unavailable_item_indices.push(index);
            if let Some(ref old_thumbnail_path) = item.thumbnail_path {
                let _ = std::fs::remove_file(old_thumbnail_path);
            }
            for album_index in item.albums.iter() {
                self.albums[*album_index].remove_image(unavailable_item_identifier);
            }
        }
        unavailable_item_indices.sort_by(|a, b| b.cmp(a)); // sort in decreasing order
        for unavailable_item_index in unavailable_item_indices.iter() {
            self.items_ordered.remove(*unavailable_item_index);
        }

        let mut persistent_items = Vec::new();
        for identifier in self.items_ordered.iter() {
            if let LibraryImageIdentifier::Path(ref path) = identifier {
                let item = LibraryPersistentStateItem {
                    path: path.clone(),
                    thumbnail_path: self.items[identifier].thumbnail_path.clone(),
                };
                persistent_items.push(item);
            }
        }
        let mut persistent_albums = Vec::new();
        for album in self.albums.iter() {
            persistent_albums.push(album.get_persistent_state());
        }
        LibraryPersistentState {
            items: persistent_items,
            albums: persistent_albums,
        }
    }

    pub fn load_persistent_state(&mut self, state: LibraryPersistentState) {
        for item in state.items {
            let identifier = self.add_item_from_path(item.path, None);
            self.items.get_mut(&identifier).unwrap().thumbnail_path = item.thumbnail_path;
        }
        for album in state.albums {
            self.albums.push(Album::from_persistent_state(album))
        }
        for album_index in 0..self.albums.len() {
            self.enumerate_album_images(album_index);
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
