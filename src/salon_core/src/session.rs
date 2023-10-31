use std::sync::Arc;

use crate::editor::Editor;
use crate::engine::{Engine, ProcessResult};
use crate::image::Image;
use crate::ir::Module;
use crate::library::{Library, LocalLibrary};
use crate::runtime::Runtime;

pub struct Session {
    pub engine: Engine,
    pub library: Box<dyn Library>,
    pub editor: Editor,
    pub state: SessionState,
}

impl Session {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let engine = Engine::new(runtime.clone());
        let library = LocalLibrary::new(runtime.clone());
        Session {
            engine,
            library: Box::new(library),
            editor: Editor::new(),
            state: SessionState::new(),
        }
    }

    pub fn set_current_image(&mut self, index: usize) {
        match self.state.current_image_index {
            Some(ref i) => {
                if *i == index {
                    return;
                }
            }
            _ => {}
        }
        self.state.current_image_index = Some(index);

        self.editor.reset_state();
        self.execute_edit();
    }

    pub fn execute_edit(&mut self) {
        if let Some(ref i) = self.state.current_image_index {
            let img = self.library.as_mut().get_image(*i);
            self.editor.execute(&mut self.engine, img);
        }
    }
}


pub struct SessionState {
    pub current_image_index: Option<usize>,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            current_image_index: None,
        }
    }
}
