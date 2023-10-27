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

    pub current_image_index: Option<usize>,
    pub current_process_result: Option<ProcessResult>,
}

impl Session {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let engine = Engine::new(runtime.clone());
        let library = LocalLibrary::new(runtime.clone());
        Session {
            engine,
            library: Box::new(library),
            editor: Editor::new(),
            current_image_index: None,
            current_process_result: None,
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

        self.editor.reset_state();

        let mut module = Module::new_trivial();
        module.add_data_for_editor_ops();

        let result = self.engine.execute_module(&module, img);

        self.current_process_result = Some(result);
    }
}
