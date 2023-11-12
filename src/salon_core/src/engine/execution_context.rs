use crate::ir::Module;

use super::ValueStore;


pub struct ExecutionContext {
    pub value_store: ValueStore,
    pub last_module: Option<Module>,
    pub last_input_image_uuid: Option<u32>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            value_store: ValueStore::new(),
            last_module: None,
            last_input_image_uuid: None,
        }
    }
}