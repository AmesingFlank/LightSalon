use std::{io::Write, path::PathBuf, thread::JoinHandle};

use sha256::TrySha256Digest;

use crate::{editor::Edit, session::Session};

pub struct EditWriterService {
    request_sender: std::sync::mpsc::Sender<Request>,
    worker_join_handle: Option<JoinHandle<()>>,
}

impl EditWriterService {
    pub fn new() -> Self {
        let (request_sender, request_receiver) = std::sync::mpsc::channel();
        let worker_join_handle = Some(std::thread::spawn(move || {
            let mut worker = Worker::new(request_receiver);
            worker.run();
        }));

        Self {
            request_sender,
            worker_join_handle,
        }
    }

    pub fn request_update(&self, edit: Edit, original_image_path: PathBuf) {
        let _ = self
            .request_sender
            .send(Request::Write(edit, original_image_path));
    }

    pub fn get_edit_path_for_image_path(image_path: &PathBuf) -> Option<PathBuf> {
        if let Ok(digest_str) = image_path.digest() {
            if let Some(storage_dir) = Session::get_persistent_storage_dir() {
                let full_path = storage_dir
                    .join("library")
                    .join(digest_str)
                    .join("edit.json");
                return Some(full_path);
            }
        }
        None
    }
}

impl Drop for EditWriterService {
    fn drop(&mut self) {
        let stop_send_result = self.request_sender.send(Request::Stop);
        if let Ok(_) = stop_send_result {
            if let Some(handle) = self.worker_join_handle.take() {
                let _ = handle.join();
            }
        }
    }
}

struct Worker {
    request_receiver: std::sync::mpsc::Receiver<Request>,
}

impl Worker {
    fn new(request_receiver: std::sync::mpsc::Receiver<Request>) -> Self {
        Self { request_receiver }
    }

    fn run(&mut self) {
        loop {
            let req = self.request_receiver.recv();
            if let Ok(req) = req {
                match req {
                    Request::Stop => {
                        break;
                    }
                    Request::Write(edit, original_image_path) => {
                        self.write(edit, original_image_path);
                    }
                }
            } else {
                break;
            }
        }
    }

    fn write(&mut self, edit: Edit, original_image_path: PathBuf) {
        if let Ok(edit_json_str) = serde_json::to_string_pretty(&edit) {
            if let Some(edit_path) =
                EditWriterService::get_edit_path_for_image_path(&original_image_path)
            {
                if let Ok(_) = std::fs::create_dir_all(edit_path.parent().unwrap()) {
                    let _ = std::fs::write(&edit_path, edit_json_str);
                }
            }
        }
    }
}

enum Request {
    Stop,
    Write(Edit, PathBuf),
}
