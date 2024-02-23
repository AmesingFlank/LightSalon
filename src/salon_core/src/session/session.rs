use std::collections::HashMap;
use std::sync::Arc;

use crate::editor::{Edit, EditHistory, Editor};
use crate::library::{Library, LibraryImageIdentifier};
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
            library: Library::new(runtime.clone(), toolbox.clone()),
            editor: Editor::new(runtime.clone(), toolbox.clone()),
            state: SessionState::new(),
            toolbox,
            runtime,
        }
    }

    pub fn set_current_image(&mut self, identifier: LibraryImageIdentifier) {
        match self.state.current_image_identifier {
            Some(ref i) => {
                if *i == identifier {
                    return;
                }
            }
            _ => {}
        }

        if let Some(ref curr_id) = self.state.current_image_identifier {
            let edit_history = self.editor.clone_edit_history();
            if edit_history.len() > 0 {
                if !(edit_history.len() == 1 && edit_history[0] == Edit::trivial()) {
                    self.state
                        .library_images_edit_histories
                        .insert(curr_id.clone(), edit_history);
                }
            }
        }

        self.editor.current_input_image = Some(self.library.get_image_from_identifier(&identifier));
        self.state.current_image_identifier = Some(identifier.clone());

        if let Some(history) = self.state.library_images_edit_histories.get(&identifier) {
            self.editor.set_edit_history(history.clone());
        } else {
            self.editor.clear_edit_history();
        }

        self.editor.execute_current_edit();
    }
}


#[derive(serde::Deserialize, serde::Serialize)]
pub struct SessionState {
    pub current_image_identifier: Option<LibraryImageIdentifier>,
    library_images_edit_histories: HashMap<LibraryImageIdentifier, EditHistory>,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            current_image_identifier: None,
            library_images_edit_histories: HashMap::new(),
        }
    }
}
