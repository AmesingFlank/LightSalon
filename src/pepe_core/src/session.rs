use std::sync::Arc;

use crate::engine::Engine;
use crate::image::Image;
use crate::library::{self, Library, LocalLibrary};
use crate::runtime::Runtime;

pub struct Session {
    pub engine: Engine,
    pub library: Arc<dyn Library>,
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
            library: Arc::new(library),
            working_image_history: Vec::new(),
        }
    }
}
