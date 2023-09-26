use std::sync::Arc;

use crate::image::Image;

pub enum Value {
    Image(Arc<Image>)
}
