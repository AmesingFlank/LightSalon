use std::sync::Arc;

use crate::engine::Engine;
use crate::image::Image;
use crate::library::{self, Library, LocalLibrary};
use crate::runtime::Runtime;

pub struct Session {
    pub engine: Engine,
    pub library: Box<dyn Library>,

    pub current_image_index: Option<usize>,
    pub working_image_history: Vec<Arc<Image>>,
    pub exposure_val: f32,
}

impl Session {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let engine = Engine::new(runtime.clone());
        let library = LocalLibrary::new(runtime.clone());
        Session {
            engine,
            library: Box::new(library),
            current_image_index: None,
            working_image_history: Vec::new(),
            exposure_val: 0.0,
        }
    }

    pub fn set_current_image(&mut self, index: usize) {
        match self.current_image_index {
            Some(ref i) => {
                if i.clone() == index {
                    return;
                }
            }
            _ => {}
        }
        self.current_image_index = Some(index);
        let img = self.library.as_mut().get_image(index);
        self.working_image_history.clear();
        self.working_image_history.push(img);
    }
}
