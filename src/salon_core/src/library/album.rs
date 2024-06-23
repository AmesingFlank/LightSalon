use std::io::Write;
use std::path::Path;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use sha256::TrySha256Digest;

use super::LibraryImageIdentifier;
use crate::runtime::{ColorSpace, Image, ImageReaderJpeg, Toolbox};
use crate::runtime::{ImageFormat, Runtime};
use crate::session::Session;
use crate::utils::uuid::{get_next_uuid, Uuid};

#[cfg(not(target_arch = "wasm32"))]
use notify::{RecursiveMode, Watcher};
#[cfg(not(target_arch = "wasm32"))]
use notify_debouncer_full::{new_debouncer, DebouncedEvent};

#[derive(PartialEq, Eq, Hash, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AlbumPersistentState {
    pub name: String,
    pub directory: Option<PathBuf>,
    // images that are not in directory
    pub additional_images: Vec<LibraryImageIdentifier>,
}

pub struct Album {
    pub name: String,
    pub directory: Option<PathBuf>,
    pub additional_images: Vec<LibraryImageIdentifier>,

    pub(super) all_images_ordered: Vec<LibraryImageIdentifier>,
    pub(super) all_images_indices: HashMap<LibraryImageIdentifier, usize>,

    #[cfg(not(target_arch = "wasm32"))]
    notify_debouncer: Option<
        notify_debouncer_full::Debouncer<
            notify::RecommendedWatcher,
            notify_debouncer_full::FileIdMap,
        >,
    >,

    #[cfg(not(target_arch = "wasm32"))]
    pub file_events_receiver:
        Option<std::sync::mpsc::Receiver<Result<Vec<DebouncedEvent>, Vec<notify::Error>>>>,
}

impl Album {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(
        name: String,
        directory: Option<PathBuf>,
        additional_images: Vec<LibraryImageIdentifier>,
    ) -> Self {
        let mut notify_debouncer = None;
        let mut file_events_receiver = None;

        let (tx, rx) = std::sync::mpsc::channel();
        // no specific tickrate, max debounce time 2 seconds
        if let Ok(mut debouncer) = new_debouncer(std::time::Duration::from_secs(1), None, tx) {
            if let Some(ref path) = directory {
                let _ = debouncer.watcher().watch(path, RecursiveMode::Recursive);
            }
            for image_identifier in additional_images.iter() {
                if let LibraryImageIdentifier::Path(ref path) = image_identifier {
                    let _ = debouncer.watcher().watch(path, RecursiveMode::Recursive);
                }
            }

            notify_debouncer = Some(debouncer);
            file_events_receiver = Some(rx);
        }
        Self {
            name,
            directory,
            additional_images,
            all_images_ordered: Vec::new(),
            all_images_indices: HashMap::new(),

            notify_debouncer,
            file_events_receiver,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(
        name: String,
        directory: Option<PathBuf>,
        additional_images: Vec<LibraryImageIdentifier>,
    ) -> Self {
        Self {
            name,
            directory,
            additional_images,
            all_images_ordered: Vec::new(),
            all_images_indices: HashMap::new(),
        }
    }

    pub fn remove_image(&mut self, image: &LibraryImageIdentifier) {
        if let Some(index) = self.additional_images.iter().position(|x| *x == *image) {
            self.additional_images.remove(index);
        }
        if let Some(index) = self.all_images_indices.get(image) {
            self.all_images_ordered.remove(*index);
            self.all_images_indices.remove(image);
        }
    }

    pub fn get_persistent_state(&self) -> AlbumPersistentState {
        AlbumPersistentState {
            name: self.name.clone(),
            directory: self.directory.clone(),
            additional_images: self.additional_images.clone(),
        }
    }

    pub fn from_persistent_state(state: AlbumPersistentState) -> Self {
        Self::new(state.name, state.directory, state.additional_images)
    }

    pub fn num_images(&self) -> usize {
        self.all_images_ordered.len()
    }

    pub fn get_identifier_at_index(&mut self, index: usize) -> &LibraryImageIdentifier {
        &self.all_images_ordered[index]
    }
}
