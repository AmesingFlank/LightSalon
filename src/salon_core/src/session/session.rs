use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::editor::{Edit, EditHistory, Editor, EditorPersistentState};
use crate::library::{Library, LibraryImageIdentifier, LibraryPersistentState};
use crate::runtime::{Runtime, Toolbox};

pub struct Session {
    pub library: Library,
    pub editor: Editor,
    pub runtime: Arc<Runtime>,
    pub toolbox: Arc<Toolbox>,
}

impl Session {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let toolbox = Arc::new(Toolbox::new(runtime.clone()));
        let mut session = Session {
            library: Library::new(runtime.clone(), toolbox.clone()),
            editor: Editor::new(runtime.clone(), toolbox.clone()),
            toolbox,
            runtime,
        };
        session.on_start();
        session
    }

    pub fn set_current_image(&mut self, identifier: LibraryImageIdentifier) {
        if let Some(new_image) = self.library.get_image_from_identifier(&identifier) {
            self.editor.set_current_image(identifier, new_image.clone());
        }
    }

    fn get_persistent_state(&self) -> SessionPersistentState {
        let library_state = self.library.get_persistent_state();
        let editor_state = self.editor.get_persistent_state();
        SessionPersistentState {
            library_state,
            editor_state,
        }
    }

    fn load_persistant_state(&mut self) -> Result<bool, String> {
        if let Some(dir) = Session::get_persistent_storage_dir() {
            let path = dir.join(self.persistent_state_file_name());
            if path.exists() {
                let state_json_str = std::fs::read_to_string(&path);
                if let Err(e) = state_json_str {
                    let err_str = "failed to read persistent state json from ".to_owned()
                        + path.to_str().unwrap_or("")
                        + ": "
                        + e.to_string().as_str();
                    return Err(err_str);
                }
                let state_json_str = state_json_str.unwrap();
                let state = serde_json::from_str::<SessionPersistentState>(state_json_str.as_str());
                if let Err(e) = state {
                    return Err(
                        "failed to parse state json file: ".to_owned() + state_json_str.as_str()
                    );
                }
                let state = state.unwrap();
                self.library.load_persistent_state(state.library_state);
                self.editor.load_persistent_state(state.editor_state);
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn save_persistent_state(&self) -> Result<bool, String> {
        if let Some(dir) = Session::get_persistent_storage_dir() {
            let path = dir.join(self.persistent_state_file_name());
            if !dir.exists() {
                if let Err(e) = std::fs::create_dir_all(dir.clone()) {
                    let err_str = "failed to create persistent state dir ".to_owned()
                        + dir.to_str().unwrap_or("")
                        + ": "
                        + e.to_string().as_str();
                    return Err(err_str);
                }
            }
            let state = self.get_persistent_state();
            let state_json_str =
                serde_json::to_string_pretty(&state).expect("failed to serialize to json");
            match std::fs::write(&path, state_json_str) {
                Ok(_) => Ok(true),
                Err(e) => {
                    let err_str = "failed to write persistent state json to ".to_owned()
                        + path.to_str().unwrap_or("")
                        + ": "
                        + e.to_string().as_str();
                    Err(err_str)
                }
            }
        } else {
            Ok(false)
        }
    }

    pub fn get_persistent_storage_dir() -> Option<PathBuf> {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "LightSalon", "LightSalon") {
            let path = proj_dirs.data_local_dir().to_path_buf();
            // Win: ~\AppData\Local\LightSalon
            // Mac: ~/Library/Application\ Support/com.LightSalon.LightSalon
            Some(path)
        } else {
            None
        }
    }

    fn persistent_state_file_name(&self) -> &str {
        "session.json"
    }

    pub fn on_exit(&mut self) {
        let save_state_result = self.save_persistent_state();
        if let Err(e) = save_state_result {
            log::error!("{}", e);
        }
    }

    fn on_start(&mut self) {
        let load_state_result = self.load_persistant_state();
        if let Err(e) = load_state_result {
            log::error!("{}", e);
        }
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct SessionPersistentState {
    library_state: LibraryPersistentState,
    editor_state: EditorPersistentState,
}
