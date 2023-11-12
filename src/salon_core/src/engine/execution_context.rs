use super::ValueStore;


pub struct ExecutionContext {
    pub value_store: ValueStore,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            value_store: ValueStore::new(),
        }
    }
}