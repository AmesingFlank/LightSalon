use std::collections::HashMap;
use std::sync::Arc;

use crate::editor::{Edit, EditHistory, Editor};
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

        if let Some(curr_index) = self.state.current_image_index {
            let edit_history = self.editor.clone_edit_history();
            if edit_history.len() > 0 {
                if !(edit_history.len() == 1 && edit_history[0] == Edit::trivial()) {
                    self.state
                        .library_images_edit_histories
                        .insert(curr_index, edit_history);
                }
            }
        }

        self.state.current_image_index = Some(index);
        self.editor.current_input_image = Some(self.library.get_image(index));

        if let Some(history) = self.state.library_images_edit_histories.get(&index) {
            self.editor.set_edit_history(history.clone());
        } else {
            self.editor.clear_edit_history();
        }

        self.editor.execute_current_edit();
    }
}

pub struct SessionState {
    pub current_image_index: Option<usize>,
    library_images_edit_histories: HashMap<usize, EditHistory>,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            current_image_index: None,
            library_images_edit_histories: HashMap::new(),
        }
    }
}
