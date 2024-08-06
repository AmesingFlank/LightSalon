use std::{collections::HashMap, path::PathBuf};

use super::LibraryImageIdentifier;

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

    pub(super) additional_images: Vec<LibraryImageIdentifier>,
    pub(super) items_ordered: Vec<LibraryImageIdentifier>,
    pub(super) item_indices: HashMap<LibraryImageIdentifier, usize>,
    pub(super) items_order_dirty: bool,

    #[cfg(not(target_arch = "wasm32"))]
    _notify_debouncer: Option<
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
            items_ordered: Vec::new(),
            item_indices: HashMap::new(),
            items_order_dirty: false,

            _notify_debouncer: notify_debouncer,
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
            items_ordered: Vec::new(),
            item_indices: HashMap::new(),
            items_order_dirty: false,
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
        self.items_ordered.len()
    }

    pub fn contains_image(&self, image_identifier: &LibraryImageIdentifier) -> bool {
        self.item_indices.contains_key(image_identifier)
    }

    pub fn contains_additional_image(&self, image_identifier: &LibraryImageIdentifier) -> bool {
        self.additional_images.contains(image_identifier)
    }
}
