use std::{collections::HashMap, sync::Arc};

use crate::{
    buffer::{Buffer, BufferProperties},
    image::{Image, ImageProperties},
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

    pub fn ensure_value_at_id_is_image_of_properties(
        &mut self,
        runtime: &Runtime,
        id: Id,
        properties: &ImageProperties,
    ) -> &Arc<Image> {
        let mut needs_create_img = true;

        match self.map.get(&id) {
            None => {}
            Some(val) => match val {
                Value::Image(ref img) => {
                    if img.properties == *properties {
                        needs_create_img = false;
                    }
                }
                _ => {}
            },
        }

        if needs_create_img {
            let new_img = Arc::new(runtime.create_image_of_properties(properties.clone()));
            self.map.insert(id, Value::Image(new_img));
        }
        self.map.get(&id).unwrap().as_image()
    }

    pub fn ensure_value_at_id_is_buffer_of_properties(
        &mut self,
        runtime: &Runtime,
        id: Id,
        properties: &BufferProperties,
    ) -> &Arc<Buffer> {
        let mut needs_create_buffer = true;

        match self.map.get(&id) {
            None => {}
            Some(val) => match val {
                Value::Buffer(ref buf) => {
                    if buf.properties == *properties {
                        needs_create_buffer = false;
                    }
                }
                _ => {}
            },
        }

        if needs_create_buffer {
            let new_buf = Arc::new(runtime.create_buffer_of_properties(properties.clone()));
            self.map.insert(id, Value::Buffer(new_buf));
        }
        self.map.get(&id).unwrap().as_buffer()
    }
}
