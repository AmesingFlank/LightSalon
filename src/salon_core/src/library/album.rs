use std::{path::PathBuf, sync::Arc};

use crate::runtime::{Image, Runtime, Toolbox};

#[derive(PartialEq, Eq, Hash, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AlbumProperties {
    pub name: String,
    pub directory: Option<PathBuf>
}

pub struct Album {
    pub properties: AlbumProperties,
    // items: HashMap<LibraryImageIdentifier, LibraryItem>,
    // item_indices: HashMap<LibraryImageIdentifier, usize>,
    // items_ordered: Vec<LibraryImageIdentifier>,
    num_temp_images: usize,
    runtime: Arc<Runtime>,
    toolbox: Arc<Toolbox>,
}

struct AlbumItem {
    image: Option<Arc<Image>>,
    thumbnail: Option<Arc<Image>>,
    thumbnail_path: Option<PathBuf>,
}
