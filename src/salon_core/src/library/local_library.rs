use std::{path::PathBuf, collections::HashMap, sync::Arc};

use crate::{image::{Image, ColorSpace}, runtime::{Runtime, ToolBox, ColorSpaceConverter}};

use super::{AddImageResult, Library};


pub struct LocalLibrary {
    paths: Vec<PathBuf>,
    images: HashMap<usize, Arc<Image>>,
    runtime: Arc<Runtime>,
    color_space_converter: ColorSpaceConverter,
}

impl LocalLibrary {
    pub fn new(runtime: Arc<Runtime>, toolbox: Arc<ToolBox>) -> Self {
        let color_space_converter = ColorSpaceConverter::new(runtime.clone());
        Self {
            paths: Vec::new(),
            images: HashMap::new(),
            runtime,
            color_space_converter,
        }
    }
}

impl Library for LocalLibrary {
    fn num_images(&self) -> usize {
        self.paths.len() as usize
    }
    fn add(&mut self, resource: &str) -> AddImageResult {
        let pathbuf = PathBuf::from(resource);
        if !std::path::Path::exists(&pathbuf.as_path()) {
            return AddImageResult::Error("cannot open ".to_owned() + resource);
        }
        for i in 0..self.paths.len() {
            if self.paths[i] == pathbuf {
                return AddImageResult::ImageAlreadyExists(i);
            }
        }
        let i = self.num_images();
        self.paths.push(pathbuf);
        AddImageResult::AddedNewImage(i)
    }
    fn get_image(&mut self, index: usize) -> Arc<Image> {
        let existing = self.images.get(&index);
        match existing {
            Some(img) => img.clone(),
            None => {
                let path = &self.paths[index];
                let img = self.runtime.create_image_from_path(path).unwrap();
                let mut img = Arc::new(img);
                img = self.color_space_converter.convert(img, ColorSpace::LinearRGB);
                self.images.insert(index, img.clone());
                img.clone()
            }
        }
    }
    fn get_thumbnail(&mut self, index: usize) -> Arc<Image> {
        self.get_image(index)
    }
}
