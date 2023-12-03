use std::sync::Arc;

use crate::runtime::{MipmapGenerator, Runtime};

pub struct Toolbox {
    pub mipmap_generator: MipmapGenerator,
}

impl Toolbox {
    pub fn new(runtime: Arc<Runtime>) -> Self{
        Self {
            mipmap_generator: MipmapGenerator::new(runtime)
        }
    }
}