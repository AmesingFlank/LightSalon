use std::io::Write;
use std::path::Path;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use sha256::TrySha256Digest;

use crate::runtime::{ColorSpace, Image, ImageReaderJpeg, Toolbox};
use crate::runtime::{ImageFormat, Runtime};
use crate::session::Session;
use crate::utils::uuid::{get_next_uuid, Uuid};

use super::LibraryImageIdentifier;

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
}

impl Album {
    pub fn get_persistent_state(&self) -> AlbumPersistentState {
        AlbumPersistentState {
            name: self.name.clone(),
            directory: self.directory.clone(),
            additional_images: self.additional_images.clone(),
        }
    }

    pub fn from_persistent_state(state: AlbumPersistentState) -> Self {
        Self {
            name: state.name,
            directory: state.directory,
            additional_images: state.additional_images,
        }
    }
}
