use std::{path::PathBuf, sync::Arc};

use crate::runtime::{Image, Runtime, Toolbox};

#[derive(PartialEq, Eq, Hash, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AlbumProperties {
    pub name: String,
    pub directory: Option<PathBuf>
}

pub struct Album {
    pub properties: AlbumProperties,
    pub images: Vec<u32>,
}
