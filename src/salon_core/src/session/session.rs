use std::sync::Arc;

use crate::editor::Editor;
use crate::engine::Engine;
use crate::library::{Library, LocalLibrary};
use crate::runtime::Runtime;

pub struct Session {
    pub library: Box<dyn Library>,
    pub editor: Editor,
    pub state: SessionState,
}

impl Session {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let library = LocalLibrary::new(runtime.clone());
        Session {
            library: Box::new(library),
            editor: Editor::new(runtime.clone()),
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

        self.editor.clear_edit_history();
        self.editor.current_input_image = Some(self.library.as_mut().get_image(index));
        self.editor.execute_current_edit();
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
