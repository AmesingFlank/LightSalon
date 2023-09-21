use std::sync::Arc;

use crate::runtime::Runtime;

pub struct Engine {
    pub runtime: Arc<Runtime>,
}