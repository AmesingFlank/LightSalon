use std::collections::HashMap;

use crate::ir::{Id, Value};

pub struct ValueStore {
    pub map: HashMap<Id, Value>,
}

impl ValueStore {
    pub fn new() -> Self {
        ValueStore {
            map: HashMap::new()
        }
    }
}