use std::{io::Write, path::PathBuf, sync::Arc, thread::JoinHandle};

use image::{DynamicImage, GenericImageView};
use sha256::TrySha256Digest;

use crate::{
    library::is_supported_image_file,
    runtime::{ColorSpace, Image, ImageFormat, ImageReaderJpeg, Runtime, Toolbox},
    session::Session,
};

/**
 * This service can either
 * 1.   Generate a thumbnail given an `Arc<Image>` and its original path, and write the generated thumbnail
 *      (whose path is comptuted based on the path of the original image)
 * OR
 * 2.   Generate a thumbnail given the path of the original image alone.
 *
 * For 1, we compute the thumbnail `Arc<Image>` immediately and return it. The image is also sent to a `WriteWorker` for saving to filesystem.
 *
 * For 2, we send the path of the original image to a `GenerateFromPath` worker, which will take care of generating and writing the thumbnail.
 *
 * In general,
 * 1 is used for "occasional" tasks of updating a thumbnail, e.g., when an image is editted.
 * 2 is used for mass generation of thumbnails for a folder of images (e.g. an album)
 */
pub struct ThumbnailGeneratorService {
    runtime: Arc<Runtime>,
    toolbox: Arc<Toolbox>,

    #[cfg(not(target_arch = "wasm32"))]
    write_request_sender: std::sync::mpsc::Sender<WriteRequest>,
    #[cfg(not(target_arch = "wasm32"))]
    write_worker_join_handle: Option<JoinHandle<()>>,

    #[cfg(not(target_arch = "wasm32"))]
    generate_from_path_request_sender: std::sync::mpsc::Sender<GenerateFromPathRequest>,
    #[cfg(not(target_arch = "wasm32"))]
    generate_from_path_worker_join_handle: Option<JoinHandle<()>>,
}

impl ThumbnailGeneratorService {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(runtime: Arc<Runtime>, toolbox: Arc<Toolbox>) -> Self {
        let (write_request_sender, write_request_receiver) = std::sync::mpsc::channel();
        let write_worker_runtime = runtime.clone();
        let write_worker_toolbox = toolbox.clone();
        let write_worker_join_handle = Some(std::thread::spawn(move || {
            let mut worker = WriteWorker::new(
                write_worker_runtime,
                write_worker_toolbox,
                write_request_receiver,
            );
            worker.run();
        }));

        let (generate_from_path_request_sender, generate_from_path_request_receiver) =
            std::sync::mpsc::channel();
        let generate_from_path_worker_join_handle = Some(std::thread::spawn(move || {
            let mut worker = GenerateFromPathWorker::new(generate_from_path_request_receiver);
            worker.run();
        }));

        Self {
            runtime,
            toolbox,
            write_request_sender,
            write_worker_join_handle,
            generate_from_path_worker_join_handle,
            generate_from_path_request_sender,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(runtime: Arc<Runtime>, toolbox: Arc<Toolbox>) -> Self {
        Self { runtime, toolbox }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn request_thumbnail_for_image_at_path(&self, image_path: PathBuf) {
        let _ = self
            .generate_from_path_request_sender
            .send(GenerateFromPathRequest::Generate(image_path));
    }

    pub fn generate_and_maybe_save_thumbnail_for_image(
        &self,
        image: Arc<Image>,
        image_original_path: Option<PathBuf>,
    ) -> Arc<Image> {
        let thumbnail_image = self.compute_thumbnail(image);

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(image_original_path) = image_original_path {
            if let Some(thumbnail_path) =
                ThumbnailGeneratorService::get_thumbnail_path_for_image_path(&image_original_path)
            {
                let _ = self.write_request_sender.send(WriteRequest::Write(
                    thumbnail_image.clone(),
                    thumbnail_path,
                    image_original_path,
                ));
            }
        }
        thumbnail_image
    }

    fn compute_thumbnail(&self, image: Arc<Image>) -> Arc<Image> {
        let image = self.toolbox.convert_color_space(image, ColorSpace::sRGB);
        let image = self
            .toolbox
            .convert_image_format(image, ImageFormat::Rgba8Unorm);
        let factor = ThumbnailGeneratorService::THUMBNAIL_MIN_DIMENSION_SIZE
            / (image.properties.dimensions.0).min(image.properties.dimensions.1) as f32;
        if factor < 0.5 {
            self.toolbox.generate_mipmap(&image);
            let thumbnail = self.toolbox.resize_image(image, factor);
            self.toolbox.generate_mipmap(&thumbnail);
            thumbnail
        } else {
            self.toolbox.generate_mipmap(&image);
            image
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
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

    const THUMBNAIL_MIN_DIMENSION_SIZE: f32 = 400.0;
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for ThumbnailGeneratorService {
    fn drop(&mut self) {
        let stop_send_result = self.write_request_sender.send(WriteRequest::Stop);
        if let Ok(_) = stop_send_result {
            if let Some(handle) = self.write_worker_join_handle.take() {
                let _ = handle.join();
            }
        }

        let stop_send_result = self
            .generate_from_path_request_sender
            .send(GenerateFromPathRequest::Stop);
        if let Ok(_) = stop_send_result {
            if let Some(handle) = self.generate_from_path_worker_join_handle.take() {
                let _ = handle.join();
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
enum WriteRequest {
    Write(Arc<Image>, PathBuf, PathBuf),
    Stop,
}

#[cfg(not(target_arch = "wasm32"))]
struct WriteWorker {
    runtime: Arc<Runtime>,
    toolbox: Arc<Toolbox>,
    request_receiver: std::sync::mpsc::Receiver<WriteRequest>,
}

#[cfg(not(target_arch = "wasm32"))]
impl WriteWorker {
    fn new(
        runtime: Arc<Runtime>,
        toolbox: Arc<Toolbox>,
        request_receiver: std::sync::mpsc::Receiver<WriteRequest>,
    ) -> Self {
        Self {
            runtime,
            toolbox,
            request_receiver,
        }
    }

    fn run(&mut self) {
        loop {
            let req = self.request_receiver.recv();
            if let Ok(req) = req {
                match req {
                    WriteRequest::Stop => {
                        break;
                    }
                    WriteRequest::Write(thumbnail_image, thumbnail_path, original_path) => {
                        self.write(thumbnail_image, thumbnail_path, original_path);
                    }
                }
            } else {
                break;
            }
        }
    }

    fn write(
        &mut self,
        thumbnail_image: Arc<Image>,
        thumbnail_path: PathBuf,
        original_path: PathBuf,
    ) {
        let mut image_reader = ImageReaderJpeg::new(
            self.runtime.clone(),
            self.toolbox.clone(),
            thumbnail_image.clone(),
        );

        if let Ok(_) = std::fs::create_dir_all(thumbnail_path.parent().unwrap()) {
            if let Ok(mut file) = std::fs::File::create(&thumbnail_path) {
                futures::executor::block_on(async move {
                    let jpeg_data = image_reader.await_jpeg_data().await;
                    let _ = file.write_all(&jpeg_data);
                });
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
enum GenerateFromPathRequest {
    Generate(PathBuf),
    Stop,
}

#[cfg(not(target_arch = "wasm32"))]
struct GenerateFromPathWorker {
    request_receiver: std::sync::mpsc::Receiver<GenerateFromPathRequest>,
}

#[cfg(not(target_arch = "wasm32"))]
impl GenerateFromPathWorker {
    fn new(
        request_receiver: std::sync::mpsc::Receiver<GenerateFromPathRequest>,
    ) -> Self {
        Self {
            request_receiver,
        }
    }

    fn run(&mut self) {
        loop {
            let req = self.request_receiver.recv();
            if let Ok(req) = req {
                match req {
                    GenerateFromPathRequest::Stop => {
                        break;
                    }
                    GenerateFromPathRequest::Generate(path) => {
                       self.generate(path);
                    }
                }
            } else {
                break;
            }
        }
    }

    fn generate(&mut self, path: PathBuf) {
        if is_supported_image_file(&path) {
            if let Some(thumbnail_path) =
                ThumbnailGeneratorService::get_thumbnail_path_for_image_path(&path)
            {
                if thumbnail_path.exists() {
                    // don't regenerate if the thumbnail already exists
                    // (e.g. we might be generating an image from a large album, but one of the images has already been editted and has an updated thumbnail)
                    return;
                }
                if let Ok(image_bytes) = std::fs::read(&path) {
                    if let Ok(img) = Runtime::create_dynamic_image_from_bytes_jpg_png(&image_bytes)
                    {
                        if let Ok(_) = std::fs::create_dir_all(thumbnail_path.parent().unwrap()) {
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
                                    let _ = file.write_all(&jpeg_data);
                                } else {
                                    let thumbnail_width = (img.width() as f32 * factor) as u32;
                                    let thumbnail_height = (img.height() as f32 * factor) as u32;
                                    let thumbnail_img = img.resize(
                                        thumbnail_width,
                                        thumbnail_height,
                                        image::imageops::FilterType::Triangle,
                                    );
                                    let jpeg_data = Self::encode_dynamic_image(&thumbnail_img);
                                    let _ = file.write_all(&jpeg_data);
                                }
                            }
                        }
                    }
                }
            };
        }
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
