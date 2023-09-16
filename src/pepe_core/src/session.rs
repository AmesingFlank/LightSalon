use std::sync::Arc;

use crate::engine::Engine;
use crate::image::Image;

pub struct Session {
    pub engine: Engine,
    pub working_image_history: Vec<Arc<Image>>,
}
