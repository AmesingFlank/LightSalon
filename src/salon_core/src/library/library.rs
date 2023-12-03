use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::image::{ColorSpace, Image};
use crate::runtime::{Runtime};

pub trait Library {
    fn num_images(&self) -> usize;
    fn add(&mut self, resource: &str) -> AddImageResult;
    fn get_image(&mut self, index: usize) -> Arc<Image>;
    fn get_thumbnail(&mut self, index: usize) -> Arc<Image>;
}

pub enum AddImageResult {
    AddedNewImage(usize),
    ImageAlreadyExists(usize),
    Error(String),
}
