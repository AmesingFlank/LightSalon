use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::editor::{Edit, EditHistory, Editor};
use crate::library::{Library, LibraryImageIdentifier, LibraryPersistentState};
use crate::runtime::{Runtime, Toolbox};

pub struct Session {
    pub library: Library,
    pub editor: Editor,
    pub runtime: Arc<Runtime>,
    pub toolbox: Arc<Toolbox>,
    state: SessionState,
}

impl Session {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let toolbox = Arc::new(Toolbox::new(runtime.clone()));
        let mut session = Session {
            library: Library::new(runtime.clone(), toolbox.clone()),
            editor: Editor::new(runtime.clone(), toolbox.clone()),
            state: SessionState::new(),
            toolbox,
            runtime,
        };
        session.on_start();
        session
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

    fn get_persistent_state(&self) -> SessionPersistentState {
        let library_state = self.library.get_persistent_state();
        SessionPersistentState { library_state }
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

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct SessionPersistentState {
    pub library_state: LibraryPersistentState,
}
