use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};

use lru::LruCache;

use crate::runtime::Image;

use super::LibraryImageIdentifier;

pub struct ImageCache {
    path_images_cache: lru::LruCache<LibraryImageIdentifier, Arc<Image>>,

    // temp images don't have a corresponding path, so we should always keep them alive instead of putting them in a cache
    temp_images: HashMap<LibraryImageIdentifier, Arc<Image>>,
}

impl ImageCache {
    pub fn new(size: usize) -> Self {
        Self {
            path_images_cache: lru::LruCache::new(NonZeroUsize::new(size).unwrap()),
            temp_images: HashMap::new(),
        }
    }

    pub fn get(&mut self, identifier: &LibraryImageIdentifier) -> Option<Arc<Image>> {
        if identifier.is_temp() {
            self.temp_images.get(&identifier).map(|v| v.clone())
        } else {
            self.path_images_cache.get(&identifier).map(|v| v.clone())
        }
    }

    pub fn set(&mut self, identifier: LibraryImageIdentifier, image: Arc<Image>) {
        if identifier.is_temp() {
            self.temp_images.insert(identifier, image);
        } else {
            self.path_images_cache.push(identifier, image);
        }
    }

    pub fn contains(&mut self, identifier: &LibraryImageIdentifier) -> bool {
        if identifier.is_temp() {
            self.temp_images.contains_key(&identifier)
        } else {
            self.path_images_cache.contains(&identifier)
        }
    }

    pub fn remove(&mut self, identifier: &LibraryImageIdentifier) {
        if identifier.is_temp() {
            self.temp_images.remove(&identifier);
        } else {
            self.path_images_cache.pop(&identifier);
        }
    }
}
