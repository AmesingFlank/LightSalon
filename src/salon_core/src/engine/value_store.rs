use std::{collections::HashMap, sync::Arc};

use crate::{
    ir::{Id, Value},
    runtime::Runtime,
};

pub struct ValueStore {
    pub map: HashMap<Id, Value>,
}

impl ValueStore {
    pub fn new() -> Self {
        ValueStore {
            map: HashMap::new(),
        }
    }

    pub fn ensure_value_at_id_is_image_of_dimensions(
        &mut self,
        runtime: &Runtime,
        id: Id,
        dimensions: (u32, u32),
    ) {
        let mut needs_create_img = true;

        match self.map.get(&id) {
            None => {}
            Some(val) => match val {
                Value::Image(ref img) => {
                    if img.dimensions == dimensions {
                        needs_create_img = false;
                    }
                }
                _ => {}
            },
        }

        if needs_create_img {
            let new_img = runtime.create_image_of_size(dimensions);
            self.map.insert(id, Value::Image(Arc::new(new_img)));
        }
    }
}
