use std::path::PathBuf;


#[derive(Debug)]
pub struct ImageEditTest {
    pub original_image_path: PathBuf,
    pub edit_json_path: PathBuf,
    pub expected_image_path: PathBuf,
}
