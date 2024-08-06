
use std::path::PathBuf;
use std::sync::Arc;

use crate::editor::{Editor};
use crate::library::{Library, LibraryImageIdentifier};
use crate::runtime::{Runtime, Toolbox};
use crate::services::services::Services;


pub struct Session {
    pub library: Library,
    pub editor: Editor,
    pub runtime: Arc<Runtime>,
    pub toolbox: Arc<Toolbox>,
    pub services: Arc<Services>,
}

impl Session {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let toolbox = Arc::new(Toolbox::new(runtime.clone()));
        let services = Arc::new(Services::new(runtime.clone(), toolbox.clone()));
        let mut session = Session {
            library: Library::new(runtime.clone(), toolbox.clone(), services.clone()),
            editor: Editor::new(runtime.clone(), toolbox.clone(), services.clone()),
            toolbox,
            runtime,
            services,
        };
        session.on_start();
        session
    }

    pub fn set_current_image(&mut self, identifier: LibraryImageIdentifier) -> Result<(), String> {
        if let Some(new_image) = self.library.get_image_from_identifier(&identifier) {
            self.editor.set_current_image(identifier, new_image.clone());
            self.update_thumbnail_for_current_image();
            Ok(())
        } else {
            self.editor.clear_current_image();
            let mut err = "Image not found".to_owned();
            if let LibraryImageIdentifier::Path(path) = identifier {
                if let Some(path_str) = path.to_str() {
                    err = err + ": " + path_str;
                }
            }
            Err(err)
        }
    }

    pub fn update_thumbnail_for_current_image(&mut self) {
        if let Some(identifier) = self.editor.current_image_identifier() {
            if let Some(edit_context) = self.editor.current_edit_context_ref() {
                if let Some(ref curr_result) = edit_context.current_result {
                    self.library.update_thumbnail_for_editted_image(
                        &identifier,
                        curr_result.before_framing.clone(),
                    );
                }
            }
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

    pub fn on_exit(&mut self) {
        self.library.save_persistent_state();
    }

    fn on_start(&mut self) {
        self.library.load_persistent_state();
    }
}
