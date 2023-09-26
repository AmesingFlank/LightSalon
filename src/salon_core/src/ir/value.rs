use std::sync::Arc;

use crate::image::Image;

pub enum Value {
    Image(Arc<Image>),
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
            Value::Image(ref i) => true,
            _ => false,
        }
    }
}
