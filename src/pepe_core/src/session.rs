use std::sync::Arc;

use crate::engine::Engine;
use crate::image::Image;
use crate::library::{self, Library, LocalLibrary};
use crate::runtime::Runtime;

pub struct Session {
    pub engine: Engine,
    pub library: Box<dyn Library>,

    pub current_image_index: Option<u32>,
    pub working_image_history: Vec<Arc<Image>>,
}

impl Session {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let engine = Engine {
            runtime: runtime.clone(),
        };
        let library = LocalLibrary::new(runtime.clone());
        Session {
            engine,
            library: Box::new(library),
            current_image_index: None,
            working_image_history: Vec::new(),
        }
    }
}
