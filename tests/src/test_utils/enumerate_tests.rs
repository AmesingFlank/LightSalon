use std::path::PathBuf;

use super::ImageEditTest;

pub fn enumerate_tests() -> Vec<ImageEditTest> {
    let mut result = Vec::new();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("salon_tests_assets");
    enumerate_tests_in_directory(&root, &mut result);
    result
}

fn enumerate_tests_in_directory(dir: &PathBuf, tests: &mut Vec<ImageEditTest>) {
    let mut original: Option<PathBuf> = None;
    let mut edit_json: Option<PathBuf> = None;
    let mut expected: Option<PathBuf> = None;
    if dir.is_dir() {
        if let Ok(read) = std::fs::read_dir(dir) {
            for entry in read {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(name) = path.file_name() {
                            if let Some(name) = name.to_str() {
                                if name == "original.jpg" {
                                    original = Some(path);
                                } else if name == "edit.json" {
                                    edit_json = Some(path);
                                } else if name == "expected.jpg" {
                                    expected = Some(path);
                                }
                            }
                        }
                    } else if path.is_dir() {
                        enumerate_tests_in_directory(&path, tests);
                    }
                }
            }
        }
    }
    if let Some(original_image_path) = original {
        if let Some(edit_json_path) = edit_json {
            if let Some(expected_image_path) = expected {
                tests.push(ImageEditTest {
                    original_image_path,
                    edit_json_path,
                    expected_image_path,
                })
            }
        }
    }
}
