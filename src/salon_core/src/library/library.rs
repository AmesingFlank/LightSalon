use std::collections::HashSet;
use std::io::Write;
use std::path::Path;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use sha256::TrySha256Digest;

use crate::runtime::{ColorSpace, Image, ImageReaderJpeg, Toolbox};
use crate::runtime::{ImageFormat, Runtime};
use crate::services;
use crate::services::services::Services;
use crate::session::Session;
use crate::utils::uuid::{get_next_uuid, Uuid};
use crate::versioning::Version;

use super::{album, is_supported_image_file, Album, AlbumPersistentState, ImageRating};

use crate::services::thumbnail_generator::ThumbnailGeneratorService;

#[derive(PartialEq, Eq, Hash, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum LibraryImageIdentifier {
    Temp(Uuid), // images that we no longer have access to after the application closes
    Path(PathBuf),
}

impl LibraryImageIdentifier {
    pub fn get_path(&self) -> Option<PathBuf> {
        match self {
            LibraryImageIdentifier::Path(ref p) => Some(p.clone()),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct LibraryImageMetaData {
    pub name: Option<String>,
}

struct LibraryItem {
    image: Option<Arc<Image>>,
    thumbnail: Option<Arc<Image>>,
    albums: HashSet<usize>,
    metadata: LibraryImageMetaData,
    rating: ImageRating,
}

pub struct Library {
    items: HashMap<LibraryImageIdentifier, LibraryItem>,
    item_indices: HashMap<LibraryImageIdentifier, usize>,
    items_ordered: Vec<LibraryImageIdentifier>,
    items_order_dirty: bool,

    // items that cannot be found
    unavailable_items: HashSet<LibraryImageIdentifier>,

    albums: Vec<Album>,

    runtime: Arc<Runtime>,
    toolbox: Arc<Toolbox>,
    services: Arc<Services>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct LibraryPersistentState {
    version: Version,
    items: Vec<LibraryPersistentStateItem>,
    albums: Vec<AlbumPersistentState>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct LibraryPersistentStateItem {
    pub path: PathBuf,
    pub rating: ImageRating,
}

impl LibraryPersistentState {
    pub fn new() -> Self {
        Self {
            version: Version::current_build(),
            items: Vec::new(),
            albums: Vec::new(),
        }
    }
}

impl Library {
    pub fn new(runtime: Arc<Runtime>, toolbox: Arc<Toolbox>, services: Arc<Services>) -> Self {
        Self {
            items: HashMap::new(),
            item_indices: HashMap::new(),
            items_ordered: Vec::new(),
            items_order_dirty: false,
            unavailable_items: HashSet::new(),
            albums: Vec::new(),
            runtime,
            toolbox,
            services,
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

    pub fn create_new_album(&mut self, name: String) {
        self.albums.push(Album::new(name, None, Vec::new()));
    }

    pub fn delete_album(&mut self, album_index: usize) {
        // TODO: support deleting albums backed by a directory as well.
        assert!(self.albums[album_index].directory.is_none());
        self.albums.remove(album_index);
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

        let thumbnail = self
            .services
            .thumbnail_generator
            .generate_and_maybe_save_thumbnail_for_image(image.clone(), None);
        let image = self
            .toolbox
            .convert_image_format(image, ImageFormat::Rgba16Float);
        let image = self
            .toolbox
            .convert_color_space(image, ColorSpace::LinearRGB);
        let library_item = LibraryItem {
            image: Some(image),
            thumbnail: Some(thumbnail),
            albums: HashSet::new(),
            metadata,
            rating: ImageRating::new(None),
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
            albums: HashSet::new(),
            metadata,
            rating: ImageRating::new(None),
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
            self.services
                .thumbnail_generator
                .request_thumbnail_for_image_at_path(path)
        }
        identifiers
    }

    pub fn add_existing_item_into_album(&mut self, image: &LibraryImageIdentifier, album: usize) {
        if !self.albums[album].contains_image(image) {
            self.albums[album].additional_images.push(image.clone());
            if self.albums[album].items_order_dirty {
                let item_index = self.albums[album].items_ordered.len();
                self.albums[album].items_ordered.push(image.clone());
                self.albums[album]
                    .item_indices
                    .insert(image.clone(), item_index);
            } else {
                let item_index = Self::ordered_insert(
                    &self.items,
                    &mut self.albums[album].items_ordered,
                    image.clone(),
                );
                self.albums[album]
                    .item_indices
                    .insert(image.clone(), item_index);
            }
        }
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
    }

    fn handle_modified_paths(
        &mut self,
        modified_paths: Vec<PathBuf>,
        associatd_album: Option<usize>,
    ) {
        let mut removed_paths = Vec::new();
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
                removed_paths.push(path);
            }
        }

        self.handle_added_or_modified_images(&added_or_modified_images, associatd_album);
        self.handle_removed_paths(removed_paths);
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
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(thumbnail_path) =
                    ThumbnailGeneratorService::get_thumbnail_path_for_image_path(image_path)
                {
                    if thumbnail_path.exists() {
                        let _ = std::fs::remove_file(thumbnail_path);
                    }
                }
            } else {
                paths_to_add.push(image_path.clone());
            }
        }
        self.add_items_from_paths(paths_to_add, associatd_album);
    }

    fn handle_removed_paths(&mut self, removed_paths: Vec<PathBuf>) {
        let mut removed_items = HashSet::new();
        let mut potential_removed_dir_paths = Vec::new();
        for path in removed_paths {
            let identifier = LibraryImageIdentifier::Path(path.clone());
            if self.items.contains_key(&identifier) {
                removed_items.insert(identifier);
            } else {
                potential_removed_dir_paths.push(path)
            }
        }
        for identifier in self.items.keys() {
            if let LibraryImageIdentifier::Path(item_path) = identifier {
                for removed_dir_path in potential_removed_dir_paths.iter() {
                    if item_path.starts_with(removed_dir_path) {
                        removed_items.insert(identifier.clone());
                        break;
                    }
                }
            }
        }
        self.remove_items(&removed_items);
    }

    pub fn remove_image_from_album(&mut self, album_index: usize, image: &LibraryImageIdentifier) {
        if let Some(index) = self.albums[album_index]
            .additional_images
            .iter()
            .position(|x| *x == *image)
        {
            self.albums[album_index].additional_images.remove(index);
        }
        if let Some(index) = self.albums[album_index].item_indices.get(image).cloned() {
            self.albums[album_index].items_ordered.remove(index);
            self.albums[album_index].item_indices.remove(image);
        }
    }

    fn remove_items(&mut self, items_to_remove: &HashSet<LibraryImageIdentifier>) {
        let mut item_indices = Vec::new();
        for item_identifier in items_to_remove.iter() {
            let item = self.items.remove(item_identifier).unwrap();
            let index = self.item_indices.remove(item_identifier).unwrap();
            item_indices.push(index);
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(image_path) = item_identifier.get_path() {
                if let Some(thumbnail_path) =
                    ThumbnailGeneratorService::get_thumbnail_path_for_image_path(&image_path)
                {
                    if thumbnail_path.exists() {
                        let _ = std::fs::remove_file(thumbnail_path);
                    }
                }
            }
            for album_index in item.albums.iter() {
                self.remove_image_from_album(*album_index, item_identifier);
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

    fn maybe_load_thumbnail(&mut self, identifier: &LibraryImageIdentifier) -> Option<Arc<Image>> {
        if self.items[identifier].thumbnail.is_none() {
            if let Some(image_path) = identifier.get_path() {
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(thumbnail_path) =
                    ThumbnailGeneratorService::get_thumbnail_path_for_image_path(&image_path)
                {
                    if let Ok(thumbnail) = self.runtime.create_image_from_path(&thumbnail_path) {
                        let thumbnail = Arc::new(thumbnail);
                        self.toolbox.generate_mipmap(&thumbnail);
                        self.items.get_mut(identifier).unwrap().thumbnail = Some(thumbnail.clone());
                        return Some(thumbnail);
                    } else {
                        let _ = std::fs::remove_file(thumbnail_path);
                    }
                }
            }
        }
        None
    }

    // return the item or delete the identifier
    fn get_fully_loaded_item(
        &mut self,
        identifier: &LibraryImageIdentifier,
    ) -> Option<&LibraryItem> {
        // If thumbnail file exists, load that first.
        let _ = self.maybe_load_thumbnail(identifier);

        if self.items[identifier].image.is_none() {
            if let LibraryImageIdentifier::Path(ref path) = identifier {
                if let Ok(image) = self.runtime.create_image_from_path(&path) {
                    let image = Arc::new(image);

                    if self.items[identifier].thumbnail.is_none() {
                        let thumbnail = self
                            .services
                            .thumbnail_generator
                            .generate_and_maybe_save_thumbnail_for_image(
                                image.clone(),
                                Some(path.clone()),
                            );
                        let item = self.items.get_mut(identifier).unwrap();
                        item.thumbnail = Some(thumbnail);
                    }

                    let image = self
                        .toolbox
                        .convert_image_format(image, ImageFormat::Rgba16Float);
                    let image = self
                        .toolbox
                        .convert_color_space(image, ColorSpace::LinearRGB);
                    {
                        let item = self.items.get_mut(identifier).unwrap();
                        item.image = Some(image);
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
            let thumbnail = self
                .services
                .thumbnail_generator
                .generate_and_maybe_save_thumbnail_for_image(
                    self.items[identifier]
                        .image
                        .clone()
                        .expect("expecting an image"),
                    identifier.get_path(),
                );
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
        if let Some(thumbnail) = self.maybe_load_thumbnail(identifier) {
            return Some(thumbnail);
        }
        let item = self.get_fully_loaded_item(identifier)?;
        Some(item.thumbnail.as_ref().unwrap().clone())
    }

    // when an image has been editted we want to use the editted image for the thumbnail
    pub fn update_thumbnail_for_editted_image(
        &mut self,
        identifier: &LibraryImageIdentifier,
        editted_image: Arc<Image>,
    ) {
        if let Some(item) = self.items.get_mut(identifier) {
            item.thumbnail = Some(
                self.services
                    .thumbnail_generator
                    .generate_and_maybe_save_thumbnail_for_image(
                        editted_image,
                        identifier.get_path(),
                    ),
            );
        }
    }

    pub fn get_rating(&self, identifier: &LibraryImageIdentifier) -> ImageRating {
        self.items[identifier].rating.clone()
    }

    pub fn set_rating(&mut self, identifier: &LibraryImageIdentifier, rating: ImageRating) {
        self.items.get_mut(identifier).unwrap().rating = rating
    }

    pub fn get_metadata(&self, identifier: &LibraryImageIdentifier) -> LibraryImageMetaData {
        self.items[identifier].metadata.clone()
    }

    fn get_persistent_state(&mut self) -> LibraryPersistentState {
        // these items are found to be unavailable, so remove them from the library
        self.remove_items(&self.unavailable_items.clone());

        self.ensure_items_order();

        let mut persistent_items = Vec::new();
        for identifier in self.items_ordered.iter() {
            if let LibraryImageIdentifier::Path(ref path) = identifier {
                let item = LibraryPersistentStateItem {
                    path: path.clone(),
                    rating: self.items[identifier].rating.clone(),
                };
                persistent_items.push(item);
            }
        }
        let mut persistent_albums = Vec::new();
        for album in self.albums.iter() {
            persistent_albums.push(album.get_persistent_state());
        }
        LibraryPersistentState {
            version: Version::current_build(),
            items: persistent_items,
            albums: persistent_albums,
        }
    }

    fn persistent_state_file_name(&self) -> &str {
        "library.json"
    }

    pub fn save_persistent_state(&mut self) {
        if let Some(dir) = Session::get_persistent_storage_dir() {
            let path = dir.join(self.persistent_state_file_name());
            if let Ok(_) = std::fs::create_dir_all(dir.clone()) {
                let state = self.get_persistent_state();
                let state_json_str =
                    serde_json::to_string_pretty(&state).expect("failed to serialize to json");
                let _ = std::fs::write(&path, state_json_str);
            }
        }
    }

    pub fn load_persistent_state(&mut self) {
        if let Some(dir) = Session::get_persistent_storage_dir() {
            let path = dir.join(self.persistent_state_file_name());
            if path.exists() {
                if let Ok(state_json_str) = std::fs::read_to_string(&path) {
                    if let Ok(state) =
                        serde_json::from_str::<LibraryPersistentState>(state_json_str.as_str())
                    {
                        for item in state.items {
                            let identifier =
                                self.add_item_from_path_impl(item.path.clone(), None, false);
                            if let Some(loaded_item) = self.items.get_mut(&identifier) {
                                loaded_item.rating = item.rating;
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
                }
            }
        }
    }
}
