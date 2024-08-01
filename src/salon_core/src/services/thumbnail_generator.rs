use std::{
    collections::LinkedList,
    io::Write,
    path::PathBuf,
    sync::{
        mpsc::{RecvTimeoutError, TryRecvError},
        Arc,
    },
    thread::JoinHandle,
};

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
    write_worker_stop_sender: std::sync::mpsc::Sender<()>,
    #[cfg(not(target_arch = "wasm32"))]
    write_worker_join_handle: Option<JoinHandle<()>>,

    #[cfg(not(target_arch = "wasm32"))]
    generate_from_path_request_sender: std::sync::mpsc::Sender<PathBuf>,
    #[cfg(not(target_arch = "wasm32"))]
    generate_from_path_worker_stop_sender: std::sync::mpsc::Sender<()>,
    #[cfg(not(target_arch = "wasm32"))]
    loaded_thumbnail_receiver: std::sync::mpsc::Receiver<LoadedThumbnail>,
    #[cfg(not(target_arch = "wasm32"))]
    generate_from_path_worker_join_handle: Option<JoinHandle<()>>,
}

impl ThumbnailGeneratorService {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(runtime: Arc<Runtime>, toolbox: Arc<Toolbox>) -> Self {
        let (write_request_sender, write_request_receiver) = std::sync::mpsc::channel();
        let (write_worker_stop_sender, write_worker_stop_receiver) = std::sync::mpsc::channel();
        let write_worker_runtime = runtime.clone();
        let write_worker_join_handle = Some(std::thread::spawn(move || {
            let mut worker = WriteWorker::new(
                write_worker_runtime,
                write_request_receiver,
                write_worker_stop_receiver,
            );
            worker.run();
        }));

        let (generate_from_path_request_sender, generate_from_path_request_receiver) =
            std::sync::mpsc::channel();
        let (generate_from_path_worker_stop_sender, generate_from_path_worker_stop_receiver) =
            std::sync::mpsc::channel();
        let generate_from_path_worker_worker_runtime = runtime.clone();
        let (loaded_thumbnail_sender, loaded_thumbnail_receiver) = std::sync::mpsc::channel();
        let generate_from_path_worker_join_handle = Some(std::thread::spawn(move || {
            let mut worker = GenerateFromPathWorker::new(
                generate_from_path_worker_worker_runtime,
                generate_from_path_request_receiver,
                generate_from_path_worker_stop_receiver,
                loaded_thumbnail_sender,
            );
            worker.run();
        }));

        Self {
            runtime,
            toolbox,
            write_request_sender,
            write_worker_stop_sender,
            write_worker_join_handle,
            generate_from_path_request_sender,
            generate_from_path_worker_stop_sender,
            loaded_thumbnail_receiver,
            generate_from_path_worker_join_handle,
        }
    }

    pub fn poll_loaded_thumbnails(&self) -> Vec<LoadedThumbnail> {
        let mut result = Vec::new();
        while let Ok(loaded) = self.loaded_thumbnail_receiver.try_recv() {
            result.push(loaded)
        }
        result
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(runtime: Arc<Runtime>, toolbox: Arc<Toolbox>) -> Self {
        Self { runtime, toolbox }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn request_thumbnail_for_image_at_path(&self, image_path: PathBuf) {
        let _ = self.generate_from_path_request_sender.send(image_path);
    }

    pub fn generate_and_maybe_save_thumbnail_for_image(
        &self,
        image: Arc<Image>,
        image_original_path: Option<PathBuf>,
    ) -> Arc<Image> {
        let thumbnail_image = Self::compute_thumbnail(&self.toolbox, image);
        self.toolbox.generate_mipmap(&thumbnail_image);

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(image_original_path) = image_original_path {
            if let Some(thumbnail_path) =
                ThumbnailGeneratorService::get_thumbnail_path_for_image_path(&image_original_path)
            {
                let _ = self
                    .write_request_sender
                    .send(WriteRequest::new(thumbnail_image.clone(), thumbnail_path));
            }
        }
        thumbnail_image
    }

    fn compute_thumbnail(toolbox: &Toolbox, image: Arc<Image>) -> Arc<Image> {
        let image = toolbox.convert_color_space(image, ColorSpace::sRGB);
        let image = toolbox.convert_image_format(image, ImageFormat::Rgba8Unorm);
        let factor = ThumbnailGeneratorService::THUMBNAIL_MIN_DIMENSION_SIZE
            / (image.properties.dimensions.0).min(image.properties.dimensions.1) as f32;
        if factor < 0.5 {
            toolbox.generate_mipmap(&image);
            let thumbnail = toolbox.resize_image(image, factor);
            thumbnail
        } else {
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
        let stop_send_result = self.write_worker_stop_sender.send(());
        if let Ok(_) = stop_send_result {
            if let Some(handle) = self.write_worker_join_handle.take() {
                let _ = handle.join();
            }
        }

        let stop_send_result = self.generate_from_path_worker_stop_sender.send(());
        if let Ok(_) = stop_send_result {
            if let Some(handle) = self.generate_from_path_worker_join_handle.take() {
                let _ = handle.join();
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct WriteRequest {
    thumbnail_image: Arc<Image>,
    thumbnail_path: PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
impl WriteRequest {
    pub fn new(thumbnail_image: Arc<Image>, thumbnail_path: PathBuf) -> Self {
        Self {
            thumbnail_image,
            thumbnail_path,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct WriteWorker {
    runtime: Arc<Runtime>,
    toolbox: Arc<Toolbox>,
    request_receiver: std::sync::mpsc::Receiver<WriteRequest>,
    stop_receiver: std::sync::mpsc::Receiver<()>,
}

#[cfg(not(target_arch = "wasm32"))]
impl WriteWorker {
    fn new(
        runtime: Arc<Runtime>,
        request_receiver: std::sync::mpsc::Receiver<WriteRequest>,
        stop_receiver: std::sync::mpsc::Receiver<()>,
    ) -> Self {
        let toolbox = Arc::new(Toolbox::new(runtime.clone()));
        Self {
            runtime,
            toolbox,
            request_receiver,
            stop_receiver,
        }
    }

    fn run(&mut self) {
        loop {
            match self.stop_receiver.try_recv() {
                Ok(_) => {
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    break;
                }
                Err(TryRecvError::Empty) => {}
            }

            match self
                .request_receiver
                .recv_timeout(std::time::Duration::from_millis(100))
            {
                Ok(path) => self.write(path),
                Err(RecvTimeoutError::Disconnected) => {
                    break;
                }
                Err(RecvTimeoutError::Timeout) => {}
            }
        }
    }

    fn write(&mut self, write_request: WriteRequest) {
        let mut image_reader = ImageReaderJpeg::new(
            self.runtime.clone(),
            self.toolbox.clone(),
            write_request.thumbnail_image,
        );

        if let Ok(_) = std::fs::create_dir_all(write_request.thumbnail_path.parent().unwrap()) {
            if let Ok(mut file) = std::fs::File::create(&write_request.thumbnail_path) {
                futures::executor::block_on(async move {
                    let jpeg_data = image_reader.await_jpeg_data().await;
                    let _ = file.write_all(&jpeg_data);
                });
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct GenerateFromPathWorker {
    runtime: Arc<Runtime>,
    toolbox: Arc<Toolbox>,
    request_receiver: std::sync::mpsc::Receiver<PathBuf>,
    stop_receiver: std::sync::mpsc::Receiver<()>,
    loaded_thumbnail_sender: std::sync::mpsc::Sender<LoadedThumbnail>,

    requests_stack: LinkedList<PathBuf>,
}

#[cfg(not(target_arch = "wasm32"))]
impl GenerateFromPathWorker {
    fn new(
        runtime: Arc<Runtime>,
        request_receiver: std::sync::mpsc::Receiver<PathBuf>,
        stop_receiver: std::sync::mpsc::Receiver<()>,
        loaded_thumbnail_sender: std::sync::mpsc::Sender<LoadedThumbnail>,
    ) -> Self {
        let toolbox = Arc::new(Toolbox::new(runtime.clone()));
        Self {
            runtime,
            toolbox,
            request_receiver,
            stop_receiver,
            loaded_thumbnail_sender,
            requests_stack: LinkedList::new(),
        }
    }

    fn run(&mut self) {
        loop {
            match self.stop_receiver.try_recv() {
                Ok(_) => {
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    break;
                }
                Err(TryRecvError::Empty) => {}
            }

            while let Ok(path) = self.request_receiver.try_recv() {
                self.requests_stack.push_back(path)
            }

            if !self.requests_stack.is_empty() {
                let latest_request = self.requests_stack.pop_back().unwrap();
                self.load_or_generate(latest_request);
            } else {
                match self
                    .request_receiver
                    .recv_timeout(std::time::Duration::from_millis(100))
                {
                    Ok(path) => self.requests_stack.push_back(path),
                    Err(RecvTimeoutError::Disconnected) => {
                        break;
                    }
                    Err(RecvTimeoutError::Timeout) => {}
                }
            }
        }
    }

    fn load_or_generate(&mut self, path: PathBuf) {
        if is_supported_image_file(&path) {
            if let Some(thumbnail_path) =
                ThumbnailGeneratorService::get_thumbnail_path_for_image_path(&path)
            {
                if thumbnail_path.exists() {
                    // don't regenerate if the thumbnail already exists
                    if let Ok(thumbnail) = self.runtime.create_image_from_path(&thumbnail_path) {
                        self.toolbox.generate_mipmap(&thumbnail);
                        let result = LoadedThumbnail {
                            original_image_path: path,
                            original_image: None,
                            thumbnail: Arc::new(thumbnail),
                        };
                        let _ = self.loaded_thumbnail_sender.send(result);
                    }
                } else if let Ok(image_bytes) = std::fs::read(&path) {
                    if let Ok(img) = Runtime::create_dynamic_image_from_bytes_jpg_png(&image_bytes)
                    {
                        if let Ok(_) = std::fs::create_dir_all(thumbnail_path.parent().unwrap()) {
                            if let Ok(mut file) = std::fs::File::create(&thumbnail_path) {
                                let image =
                                    Arc::new(self.runtime.create_image_from_dynamic_image(img));
                                let image = self
                                    .toolbox
                                    .convert_image_format(image, ImageFormat::Rgba16Float);
                                let image = self
                                    .toolbox
                                    .convert_color_space(image, ColorSpace::LinearRGB);
                                let thumbnail_image = ThumbnailGeneratorService::compute_thumbnail(
                                    &self.toolbox,
                                    image.clone(),
                                );
                                self.toolbox.generate_mipmap(&thumbnail_image);
                                let result = LoadedThumbnail {
                                    original_image_path: path,
                                    original_image: Some(image),
                                    thumbnail: thumbnail_image.clone(),
                                };
                                let _ = self.loaded_thumbnail_sender.send(result);
                                let mut image_reader = ImageReaderJpeg::new(
                                    self.runtime.clone(),
                                    self.toolbox.clone(),
                                    thumbnail_image,
                                );
                                futures::executor::block_on(async move {
                                    let jpeg_data = image_reader.await_jpeg_data().await;
                                    let _ = file.write_all(&jpeg_data);
                                });
                            }
                        }
                    }
                }
            };
        }
    }
}

pub struct LoadedThumbnail {
    pub original_image_path: PathBuf,
    pub original_image: Option<Arc<Image>>,
    pub thumbnail: Arc<Image>,
}
