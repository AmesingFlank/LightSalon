use std::sync::Arc;

use crate::editor::Editor;
use crate::library::Library;
use crate::runtime::{Runtime, Toolbox};

pub struct Session {
    pub library: Library,
    pub editor: Editor,
    pub state: SessionState,
    pub runtime: Arc<Runtime>,
    pub toolbox: Arc<Toolbox>,
}

impl Session {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let toolbox = Arc::new(Toolbox::new(runtime.clone()));
        Session {
            library: Library::new(toolbox.clone()),
            editor: Editor::new(runtime.clone(), toolbox.clone()),
            state: SessionState::new(),
            toolbox,
            runtime,
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
        self.editor.current_input_image = Some(self.library.get_image(index));
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
