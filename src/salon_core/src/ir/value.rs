use std::sync::Arc;

use crate::runtime::{Buffer, Image};

pub enum Value {
    Image(Arc<Image>),
    Buffer(Arc<Buffer>),
}

impl Value {
    pub fn as_image(&self) -> &Arc<Image> {
        match self {
            Value::Image(ref i) => i,
            _ => {
                panic!("expecting image")
            }
        }
    }
    pub fn is_image(&self) -> bool {
        match self {
            Value::Image(_) => true,
            _ => false,
        }
    }
    pub fn as_buffer(&self) -> &Arc<Buffer> {
        match self {
            Value::Buffer(ref b) => b,
            _ => {
                panic!("expecting buffer")
            }
        }
    }
    pub fn is_buffer(&self) -> bool {
        match self {
            Value::Buffer(_) => true,
            _ => false,
        }
    }
}
