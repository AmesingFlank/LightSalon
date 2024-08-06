use std::sync::Arc;

use crate::engine::common::ImageHistogram;
use crate::runtime::{BufferReader, Image};

pub struct EditResult {
    pub final_image: Arc<Image>,
    pub geometry_only: Arc<Image>,
    pub before_framing: Arc<Image>,
    pub histogram_final: Option<BufferReader<ImageHistogram>>,
    pub masked_edit_results: Vec<MaskedEditResult>,
}

pub struct MaskedEditResult {
    pub mask: Arc<Image>,
    pub mask_terms: Vec<Arc<Image>>,
    pub result_image: Arc<Image>,
}