use std::sync::Arc;

use crate::runtime::{Runtime, Toolbox};

use super::{edit_writer::EditWriterService, thumbnail_generator::ThumbnailGeneratorService};

pub struct Services {
    pub thumbnail_generator: ThumbnailGeneratorService,

    #[cfg(not(target_arch = "wasm32"))]
    pub edit_writer: EditWriterService,
}

impl Services {
    pub fn new(runtime: Arc<Runtime>, toolbox: Arc<Toolbox>) -> Self {
        Self {
            thumbnail_generator: ThumbnailGeneratorService::new(runtime, toolbox),

            #[cfg(not(target_arch = "wasm32"))]
            edit_writer: EditWriterService::new(),
        }
    }
}
