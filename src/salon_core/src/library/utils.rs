use std::path::PathBuf;

pub fn is_supported_image_file(path: &PathBuf) -> bool {
    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            let ext_str = ext_str.to_lowercase();
            if ext_str == "jpg" || ext_str == "jpeg" || ext_str == "png" {
                return true;
            }
        }
    }
    false
}
