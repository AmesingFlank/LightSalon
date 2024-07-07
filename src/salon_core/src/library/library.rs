use std::collections::HashSet;
use std::io::Write;
use std::path::Path;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use sha256::TrySha256Digest;

use crate::runtime::{ColorSpace, Image, ImageReaderJpeg, Toolbox};
use crate::runtime::{ImageFormat, Runtime};
use crate::session::Session;
use crate::utils::uuid::{get_next_uuid, Uuid};

use super::{album, is_supported_image_file, Album, AlbumPersistentState};

use super::services::thumbnail_generator::ThumbnailGeneratorService;

#[derive(PartialEq, Eq, Hash, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum LibraryImageIdentifier {
    Temp(Uuid), // images that we no longer have access to after the application closes
    Path(PathBuf),
}

#[derive(Clone)]
pub struct LibraryImageMetaData {
    pub name: Option<String>,
}

struct LibraryItem {
    image: Option<Arc<Image>>,
    thumbnail: Option<Arc<Image>>,
    thumbnail_path: Option<PathBuf>,
    albums: HashSet<usize>,
    metadata: LibraryImageMetaData,
}

pub struct Library {
    items: HashMap<LibraryImageIdentifier, LibraryItem>,
    item_indices: HashMap<LibraryImageIdentifier, usize>,
    items_ordered: Vec<LibraryImageIdentifier>,
    items_order_dirty: bool,

    runtime: Arc<Runtime>,
    toolbox: Arc<Toolbox>,
    albums: Vec<Album>,

    // items that cannot be found
    unavailable_items: HashSet<LibraryImageIdentifier>,

    #[cfg(not(target_arch = "wasm32"))]
    thumbnail_generator: ThumbnailGeneratorService,
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
            items_order_dirty: false,
            albums: Vec::new(),
            runtime,
            toolbox,
            unavailable_items: HashSet::new(),
            #[cfg(not(target_arch = "wasm32"))]
            thumbnail_generator: ThumbnailGeneratorService::new(),
        }
    }

    pub fn num_images_total(&self) -> usize {
        self.items.len() as usize
    }

    pub fn albums(&self) -> &Vec<Album> {
        &self.albums
    }

    pub fn albums_mut(&mut self) -> &mut Vec<Album> {
        &mut self.albums
    }

    pub fn num_images_in_album(&self, album_index: usize) -> usize {
        self.albums[album_index].num_images()
    }

    // the item should already be present in all_items, this function adds it into a items_ordered vector (of either the entire library, or an album)
    fn ordered_insert(
        all_items: &HashMap<LibraryImageIdentifier, LibraryItem>,
        items_ordered: &mut Vec<LibraryImageIdentifier>,
        identifier: LibraryImageIdentifier,
    ) -> usize {
        let metadata = all_items.get(&identifier).unwrap().metadata.clone();
        for i in 0..items_ordered.len() {
            if metadata.name < all_items.get(&items_ordered[i]).unwrap().metadata.name {
                items_ordered.insert(i, identifier.clone());
                return i;
            }
        }
        items_ordered.push(identifier);
        items_ordered.len() - 1
    }

    fn add_item(
        &mut self,
        mut item: LibraryItem,
        identifier: LibraryImageIdentifier,
        album: Option<usize>,
        ensure_order: bool,
    ) {
        if let Some(album) = album {
            item.albums.insert(album);
        }
        let old_item = self.items.insert(identifier.clone(), item);
        if old_item.is_none() {
            if ensure_order {
                if self.items_order_dirty {
                    self.ensure_items_order();
                }
                let index =
                    Self::ordered_insert(&self.items, &mut self.items_ordered, identifier.clone());
                self.item_indices.insert(identifier.clone(), index);
                self.items_order_dirty = false;
            } else {
                self.item_indices
                    .insert(identifier.clone(), self.items_ordered.len());
                self.items_ordered.push(identifier.clone());
                self.items_order_dirty = true;
            }
        }

        // insert item into album
        if let Some(album) = album {
            if !self.albums[album].item_indices.contains_key(&identifier) {
                if ensure_order {
                    if self.albums[album].items_order_dirty {
                        self.ensure_items_order();
                    }
                    let index = Self::ordered_insert(
                        &self.items,
                        &mut self.albums[album].items_ordered,
                        identifier.clone(),
                    );
                    self.albums[album]
                        .item_indices
                        .insert(identifier.clone(), index);
                    self.albums[album].items_order_dirty = false;
                } else {
                    self.albums[album]
                        .item_indices
                        .insert(identifier.clone(), self.items_ordered.len());
                    self.albums[album].items_ordered.push(identifier.clone());
                    self.albums[album].items_order_dirty = true;
                }
            }
        }
    }

    fn ensure_items_order_impl(
        all_items: &HashMap<LibraryImageIdentifier, LibraryItem>,
        items_ordered: &mut Vec<LibraryImageIdentifier>,
        item_indices: &mut HashMap<LibraryImageIdentifier, usize>,
    ) {
        items_ordered.sort_by(|a, b| {
            // TODO: too many HashMap accesses?
            all_items
                .get(a)
                .unwrap()
                .metadata
                .name
                .cmp(&all_items.get(b).unwrap().metadata.name)
        });
        item_indices.clear();
        for i in 0..items_ordered.len() {
            item_indices.insert(items_ordered[i].clone(), i);
        }
    }

    fn ensure_items_order(&mut self) {
        if !self.items_order_dirty {
            return;
        }
        Self::ensure_items_order_impl(&self.items, &mut self.items_ordered, &mut self.item_indices);
        self.items_order_dirty = false;
    }

    fn ensure_items_order_for_album(&mut self, album: usize) {
        let album = &mut self.albums[album];
        if album.items_order_dirty {
            return;
        }
        Self::ensure_items_order_impl(
            &self.items,
            &mut album.items_ordered,
            &mut album.item_indices,
        );
        album.items_order_dirty = false;
    }

    pub fn add_image_temp(
        &mut self,
        image: Arc<Image>,
        album: Option<usize>,
        metadata: LibraryImageMetaData,
    ) -> LibraryImageIdentifier {
        let temp_image_id = LibraryImageIdentifier::Temp(get_next_uuid());

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
            albums: HashSet::new(),
            metadata,
        };
        self.add_item(
            library_item,
            temp_image_id.clone(),
            album,
            /* ensure_order */ true,
        );
        temp_image_id
    }

    fn add_item_from_path_impl(
        &mut self,
        path: PathBuf,
        album: Option<usize>,
        ensure_order: bool,
    ) -> LibraryImageIdentifier {
        let mut metadata = LibraryImageMetaData { name: None };
        if let Some(name) = path.file_name() {
            if let Some(name) = name.to_str() {
                metadata.name = Some(name.to_owned());
            }
        }

        let id = LibraryImageIdentifier::Path(path);

        let item = LibraryItem {
            image: None,
            thumbnail: None,
            thumbnail_path: None,
            albums: HashSet::new(),
            metadata,
        };
        self.add_item(item, id.clone(), album, ensure_order);
        id
    }

    pub fn add_single_item_from_path(
        &mut self,
        path: PathBuf,
        album: Option<usize>,
    ) -> LibraryImageIdentifier {
        self.add_item_from_path_impl(path, album, true)
    }

    pub fn add_items_from_paths(
        &mut self,
        paths: Vec<PathBuf>,
        album: Option<usize>,
    ) -> Vec<LibraryImageIdentifier> {
        let mut identifiers = Vec::new();
        for path in paths {
            let identifier = self.add_item_from_path_impl(path.clone(), album, false);
            identifiers.push(identifier);

            #[cfg(not(target_arch = "wasm32"))]
            self.thumbnail_generator.request_thumbnail_for_image(path)
        }
        identifiers
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

    pub fn poll_updates(&mut self) {
        // album events
        #[cfg(not(target_arch = "wasm32"))]
        for i in 0..self.albums.len() {
            if self.albums[i].file_events_receiver.is_none() {
                continue;
            }
            if let Ok(events_results) = self.albums[i]
                .file_events_receiver
                .as_mut()
                .unwrap()
                .try_recv()
            {
                let mut modified_paths = Vec::new();
                if let Ok(events) = events_results {
                    for event in events.iter() {
                        let mut paths = event.paths.clone();
                        modified_paths.append(&mut paths);
                    }
                }
                self.handle_modified_paths(modified_paths, Some(i));
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let thumbnail_results = self.thumbnail_generator.poll_results();
            for generated_thumbnail in thumbnail_results {
                let thumbnail_path = generated_thumbnail.thumbnail_path;
                let identifier =
                    LibraryImageIdentifier::Path(generated_thumbnail.original_image_path);
                if let Some(item) = self.items.get_mut(&identifier) {
                    if item.thumbnail_path.is_some() {
                        // then the main thread generated the thumbnail itself, disgard the one generated by the service.
                        let _ = std::fs::remove_file(&thumbnail_path);
                    } else {
                        item.thumbnail_path = Some(thumbnail_path);
                    }
                }
            }
        }
    }

    fn handle_modified_paths(
        &mut self,
        modified_paths: Vec<PathBuf>,
        associatd_album: Option<usize>,
    ) {
        let mut removed_files = Vec::new();
        let mut removed_dirs = Vec::new();

        let mut added_or_modified_images = Vec::new();

        for path in modified_paths {
            if path.exists() {
                if path.is_file() {
                    if is_supported_image_file(&path) {
                        added_or_modified_images.push(path);
                    }
                } else {
                    let mut all_images_in_path = Vec::new();
                    Self::enumerate_images_in_directory(&path, &mut all_images_in_path);
                    added_or_modified_images.append(&mut all_images_in_path);
                }
            } else {
                if path.is_file() {
                    removed_files.push(path);
                } else {
                    removed_dirs.push(path);
                }
            }
        }

        self.handle_added_or_modified_images(&added_or_modified_images, associatd_album);
        self.handle_removed_paths(&removed_files, &removed_dirs);
    }

    fn handle_added_or_modified_images(
        &mut self,
        images: &Vec<PathBuf>,
        associatd_album: Option<usize>,
    ) {
        let mut paths_to_add = Vec::new();
        for image_path in images {
            let item_identifier = LibraryImageIdentifier::Path(image_path.clone());
            if self.items.contains_key(&item_identifier) {
                let item = self.items.get_mut(&item_identifier).unwrap();
                item.image = None;
                item.thumbnail = None;
                if let Some(ref thumbnail_path) = item.thumbnail_path {
                    let _ = std::fs::remove_file(&thumbnail_path);
                }
                item.thumbnail_path = None;
            } else {
                paths_to_add.push(image_path.clone());
            }
        }
        self.add_items_from_paths(paths_to_add, associatd_album);
    }

    fn handle_removed_paths(&mut self, removed_files: &Vec<PathBuf>, removed_dirs: &Vec<PathBuf>) {
        let mut removed_items = HashSet::new();
        for path in removed_files.iter() {
            let identifier = LibraryImageIdentifier::Path(path.clone());
            if self.items.contains_key(&identifier) {
                removed_items.insert(identifier);
            }
        }
        for identifier in self.items.keys() {
            if let LibraryImageIdentifier::Path(item_path) = identifier {
                for removed_dir_path in removed_dirs.iter() {
                    if item_path.starts_with(removed_dir_path) {
                        removed_items.insert(identifier.clone());
                        break;
                    }
                }
            }
        }
        self.remove_items(&removed_items);
    }

    fn remove_image_from_album(album: &mut Album, image: &LibraryImageIdentifier) {
        if let Some(index) = album.additional_images.iter().position(|x| *x == *image) {
            album.additional_images.remove(index);
        }
        if let Some(index) = album.item_indices.get(image) {
            album.items_ordered.remove(*index);
            album.item_indices.remove(image);
        }
    }

    fn remove_items(&mut self, items_to_remove: &HashSet<LibraryImageIdentifier>) {
        let mut item_indices = Vec::new();
        for item_identifier in items_to_remove.iter() {
            let item = self.items.remove(item_identifier).unwrap();
            let index = self.item_indices.remove(item_identifier).unwrap();
            item_indices.push(index);
            if let Some(ref old_thumbnail_path) = item.thumbnail_path {
                let _ = std::fs::remove_file(old_thumbnail_path);
            }
            for album_index in item.albums.iter() {
                Self::remove_image_from_album(&mut self.albums[*album_index], item_identifier);
            }
        }
        item_indices.sort_by(|a, b| b.cmp(a)); // sort in decreasing order
        for unavailable_item_index in item_indices.iter() {
            self.items_ordered.remove(*unavailable_item_index);
        }
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
            album.items_ordered = all_images.clone();
            album.item_indices.clear();
            for i in 0..album.items_ordered.len() {
                album.item_indices.insert(album.items_ordered[i].clone(), i);
            }
        }
        let mut paths_to_add = Vec::new();
        for identifier in all_images.iter() {
            if let Some(item) = self.items.get_mut(identifier) {
                item.albums.insert(album_index);
            } else {
                match identifier {
                    LibraryImageIdentifier::Path(path) => paths_to_add.push(path.clone()),
                    _ => {
                        panic!("expecting the identifier to be a LibraryImageIdentifier::Path")
                    }
                }
            }
        }
        self.add_items_from_paths(paths_to_add, Some(album_index));
        self.ensure_items_order_for_album(album_index);
    }

    fn enumerate_images_in_directory(dir: &PathBuf, images: &mut Vec<PathBuf>) {
        if dir.is_dir() {
            if let Ok(read) = std::fs::read_dir(dir) {
                for entry in read {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_file() && is_supported_image_file(&path) {
                            images.push(path);
                        } else if path.is_dir() {
                            Self::enumerate_images_in_directory(&path, images);
                        }
                    }
                }
            }
        }
    }

    pub fn get_identifier_at_index(&mut self, index: usize) -> &LibraryImageIdentifier {
        self.ensure_items_order();
        &self.items_ordered[index]
    }

    pub fn get_identifier_at_index_for_album(
        &mut self,
        index: usize,
        album: usize,
    ) -> &LibraryImageIdentifier {
        self.ensure_items_order_for_album(album);
        &self.albums[album].items_ordered[index]
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
                    if let Some(ref old_thumbnail_path) = self.items[identifier].thumbnail_path {
                        let _ = std::fs::remove_file(old_thumbnail_path);
                        self.items.get_mut(identifier).unwrap().thumbnail_path = None;
                    }

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
            if let Some(ref thumbnail_path) = self.items[identifier].thumbnail_path {
                if let Ok(thumbnail) = self.runtime.create_image_from_path(thumbnail_path) {
                    let thumbnail = Arc::new(thumbnail);
                    self.toolbox.generate_mipmap(&thumbnail);
                    self.items.get_mut(identifier).unwrap().thumbnail = Some(thumbnail.clone());
                } else {
                    let _ = std::fs::remove_file(thumbnail_path);
                    self.items.get_mut(identifier).unwrap().thumbnail_path = None;
                }
            }

            if self.items[identifier].thumbnail_path.is_none() {
                // then we need to regenerate the thumbnail
                let image = self.items[identifier].image.as_ref().unwrap().clone();
                let image = self.toolbox.convert_color_space(image, ColorSpace::sRGB);
                let image = self
                    .toolbox
                    .convert_image_format(image, ImageFormat::Rgba8Unorm);
                let thumbnail = self.compute_thumbnail(image);
                self.items.get_mut(identifier).unwrap().thumbnail = Some(thumbnail.clone());
                if let LibraryImageIdentifier::Path(ref path) = identifier {
                    let thumbnail_path = self.save_thumbnail(thumbnail.clone(), path);
                    self.items.get_mut(identifier).unwrap().thumbnail_path = thumbnail_path;
                }
            }
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
        self.remove_items(&self.unavailable_items.clone());

        self.ensure_items_order();

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
            let identifier = self.add_item_from_path_impl(item.path.clone(), None, false);
            if item.thumbnail_path.is_some() {
                self.items.get_mut(&identifier).unwrap().thumbnail_path = item.thumbnail_path;
            } else {
                #[cfg(not(target_arch = "wasm32"))]
                self.thumbnail_generator
                    .request_thumbnail_for_image(item.path);
            }
        }
        for album in state.albums {
            self.albums.push(Album::from_persistent_state(album))
        }
        for album_index in 0..self.albums.len() {
            self.enumerate_album_images(album_index);
        }
        self.ensure_items_order();
    }

    fn compute_thumbnail(&self, image: Arc<Image>) -> Arc<Image> {
        let factor = ThumbnailGeneratorService::THUMBNAIL_MIN_DIMENSION_SIZE
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

    fn save_thumbnail(
        &self,
        thumbnail: Arc<Image>,
        original_image_path: &PathBuf,
    ) -> Option<PathBuf> {
        if let Some(thumbnail_path) =
            ThumbnailGeneratorService::get_thumbnail_path_for_image_path(original_image_path)
        {
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
