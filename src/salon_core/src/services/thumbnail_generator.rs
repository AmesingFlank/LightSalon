use std::{io::Write, path::PathBuf, thread::JoinHandle};

use image::{DynamicImage, GenericImageView};
use sha256::TrySha256Digest;

use crate::{library::is_supported_image_file, runtime::Runtime, session::Session};

pub struct ThumbnailGeneratorService {
    response_receiver: std::sync::mpsc::Receiver<ThumbnailGeneratorServiceResponse>,
    request_sender: std::sync::mpsc::Sender<ThumbnailGeneratorServiceRequest>,
    worker_join_handle: Option<JoinHandle<()>>,
}

impl ThumbnailGeneratorService {
    pub fn new() -> Self {
        let (request_sender, request_receiver) = std::sync::mpsc::channel();
        let (response_sender, response_receiver) = std::sync::mpsc::channel();

        let worker_join_handle = Some(std::thread::spawn(move || {
            let mut worker =
                ThumbnailGeneratorServiceWorker::new(request_receiver, response_sender);
            worker.run();
        }));

        Self {
            response_receiver,
            request_sender,
            worker_join_handle,
        }
    }

    pub fn poll_results(&self) -> Vec<GeneratedThumbnail> {
        let mut results = Vec::new();
        while let Ok(response) = self.response_receiver.try_recv() {
            if let ThumbnailGeneratorServiceResponse::Generated(result) = response {
                results.push(result)
            }
        }
        results
    }

    pub fn request_thumbnail_for_image(&self, image_path: PathBuf) {
        let _ = self
            .request_sender
            .send(ThumbnailGeneratorServiceRequest::Generate(image_path));
    }

    pub fn get_thumbnail_path_for_image_path(image_path: &PathBuf) -> Option<PathBuf> {
        if let Ok(digest_str) = image_path.digest() {
            if let Some(storage_dir) = Session::get_persistent_storage_dir() {
                let full_path = storage_dir
                    .join("library")
                    .join(digest_str)
                    .join("thumbnail.jpg");
                return Some(full_path);
            }
        }
        None
    }

    pub const THUMBNAIL_MIN_DIMENSION_SIZE: f32 = 400.0;
}

impl Drop for ThumbnailGeneratorService {
    fn drop(&mut self) {
        let stop_send_result = self
            .request_sender
            .send(ThumbnailGeneratorServiceRequest::Stop);
        if let Ok(_) = stop_send_result {
            if let Some(handle) = self.worker_join_handle.take() {
                let _ = handle.join();
            }
        }
    }
}

struct ThumbnailGeneratorServiceWorker {
    request_receiver: std::sync::mpsc::Receiver<ThumbnailGeneratorServiceRequest>,
    response_sender: std::sync::mpsc::Sender<ThumbnailGeneratorServiceResponse>,
}

impl ThumbnailGeneratorServiceWorker {
    fn new(
        request_receiver: std::sync::mpsc::Receiver<ThumbnailGeneratorServiceRequest>,
        response_sender: std::sync::mpsc::Sender<ThumbnailGeneratorServiceResponse>,
    ) -> Self {
        Self {
            request_receiver,
            response_sender,
        }
    }

    fn run(&mut self) {
        loop {
            let req = self.request_receiver.recv();
            if let Ok(req) = req {
                match req {
                    ThumbnailGeneratorServiceRequest::Stop => {
                        break;
                    }
                    ThumbnailGeneratorServiceRequest::Generate(path) => {
                        let response = self.generate(path);
                        let _ = self.response_sender.send(response);
                    }
                }
            } else {
                break;
            }
        }
    }

    fn generate(&mut self, path: PathBuf) -> ThumbnailGeneratorServiceResponse {
        if is_supported_image_file(&path) {
            if let Ok(image_bytes) = std::fs::read(&path) {
                if let Some(thumbnail_path) =
                    ThumbnailGeneratorService::get_thumbnail_path_for_image_path(&path)
                {
                    if let Ok(img) = Runtime::create_dynamic_image_from_bytes_jpg_png(&image_bytes)
                    {
                        if let Ok(mut file) = std::fs::File::create(&thumbnail_path) {
                            let aspect_ratio = img.width() as f32 / img.height() as f32;
                            let factor = if aspect_ratio >= 1.0 {
                                ThumbnailGeneratorService::THUMBNAIL_MIN_DIMENSION_SIZE
                                    / img.height() as f32
                            } else {
                                ThumbnailGeneratorService::THUMBNAIL_MIN_DIMENSION_SIZE
                                    / img.width() as f32
                            };

                            if factor >= 1.0 {
                                // no need to resize;
                                let jpeg_data = Self::encode_dynamic_image(&img);
                                if let Ok(_) = file.write_all(&jpeg_data) {
                                    return ThumbnailGeneratorServiceResponse::Generated(
                                        GeneratedThumbnail {
                                            original_image_path: path,
                                            thumbnail_path,
                                        },
                                    );
                                }
                            } else {
                                let thumbnail_width = (img.width() as f32 * factor) as u32;
                                let thumbnail_height = (img.height() as f32 * factor) as u32;
                                let thumbnail_img = img.resize(
                                    thumbnail_width,
                                    thumbnail_height,
                                    image::imageops::FilterType::Triangle,
                                );
                                let jpeg_data = Self::encode_dynamic_image(&thumbnail_img);
                                if let Ok(_) = file.write_all(&jpeg_data) {
                                    return ThumbnailGeneratorServiceResponse::Generated(
                                        GeneratedThumbnail {
                                            original_image_path: path,
                                            thumbnail_path,
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
            };
        }
        ThumbnailGeneratorServiceResponse::CouldNotGenerate(path)
    }

    fn encode_dynamic_image(image: &DynamicImage) -> Vec<u8> {
        let image_buffer = image.to_rgba8();
        let mut jpeg: Vec<u8> = Vec::new();
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 100);
        encoder
            .encode(
                &image_buffer,
                image.width(),
                image.height(),
                image::ColorType::Rgba8,
            )
            .expect("Failed to encode image into jpeg");
        jpeg
    }
}

pub struct GeneratedThumbnail {
    pub original_image_path: PathBuf,
    pub thumbnail_path: PathBuf,
}

enum ThumbnailGeneratorServiceResponse {
    Generated(GeneratedThumbnail),
    CouldNotGenerate(PathBuf),
}

enum ThumbnailGeneratorServiceRequest {
    Generate(PathBuf),
    Stop,
}
