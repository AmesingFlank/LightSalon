use std::{path::PathBuf, collections::HashMap, sync::Arc};

use crate::image::Image;
use crate::runtime::Runtime;

pub trait Library {
    fn num_images(&self) -> u32;
    fn add(&mut self, resource: &str) -> Result<(), String>;
    fn get_image(&mut self, index: u32) -> Arc<Image>;
}

pub struct LocalLibrary {
    paths: Vec<PathBuf>,
    images: HashMap<u32, Arc<Image>>,
    runtime: Arc<Runtime>,
}

impl LocalLibrary {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Self {
            paths: Vec::new(),
            images: HashMap::new(),
            runtime,
        }
    }
}

impl Library for LocalLibrary {
    fn num_images(&self) -> u32 {
        self.paths.len() as u32
    }
    fn add(&mut self, resource: &str) -> Result<(), String> {
        let pathbuf = PathBuf::from(resource);
        if !std::path::Path::exists(&pathbuf.as_path()) {
            return Err("cannot open ".to_owned()+resource)
        }
        self.paths.push(pathbuf);
        Ok(())
    }
    fn get_image(&mut self, index: u32) -> Arc<Image> {
        let existing = self.images.get(&index);
        match existing {
            Some(img) => img.clone(),
            None => {
                let path = &self.paths[index as usize];
                let img = Image::create_from_path(self.runtime.as_ref(), path).unwrap();
                let img = Arc::new(img);
                self.images.insert(index, img.clone());
                img.clone()
            }
        }
    }
}
