use super::{AddedImageOrAlbum, AppUiState};
use eframe::{
    egui::{self, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::{
    library::{LibraryImageIdentifier, LibraryImageMetaData},
    runtime::{ColorSpace, ImageFormat, ImageReaderJpeg, Runtime, Toolbox},
    session::Session,
};
use std::{future::Future, ops::Add, sync::Arc};

#[cfg(not(target_arch = "wasm32"))]
pub fn file_dialogue_export_image(
    context: egui::Context,
    session: &mut Session,
    ui_state: &mut AppUiState,
) {
    session.editor.commit_transient_edit(false);
    let final_image = ui_state
        .export_image_selected_resolution
        .as_ref()
        .unwrap()
        .clone();
    let final_image = session
        .toolbox
        .convert_color_space(final_image, ColorSpace::sRGB);
    let final_image = session
        .toolbox
        .convert_image_format(final_image, ImageFormat::Rgba8Unorm);
    let mut image_reader = ImageReaderJpeg::new(
        session.runtime.clone(),
        session.toolbox.clone(),
        final_image,
    );

    let mut task = rfd::AsyncFileDialog::new().add_filter("extension", &["jpg"]);
    if let Some(name) = session
        .library
        .get_metadata(&session.editor.current_image_identifier().unwrap())
        .name
    {
        task = task.set_file_name(
            ui_state
                .export_file_name
                .clone()
                .expect("expecting an export file name"),
        );
    }
    let file_handle = task.save_file();
    execute(async move {
        let file = file_handle.await;
        let jpeg_data = image_reader.await_jpeg_data().await;
        if let Some(file) = file {
            file.write(&jpeg_data).await.expect("Write file failed");
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn file_dialogue_export_edit(
    context: egui::Context,
    session: &mut Session,
    ui_state: &mut AppUiState,
) {
    let edit = session.editor.get_full_size_edit();
    let edit_json_str = serde_json::to_string_pretty(&edit).expect("failed to serialize to json");

    let mut task = rfd::AsyncFileDialog::new().add_filter("extension", &["json"]);
    if let Some(name) = session
        .library
        .get_metadata(&session.editor.current_image_identifier().unwrap())
        .name
    {
        task = task.set_file_name(edit_json_file_name(&name));
    }
    let file_handle = task.save_file();
    execute(async move {
        let file = file_handle.await;
        if let Some(file) = file {
            file.write(edit_json_str.as_bytes())
                .await
                .expect("Write file failed");
        }
    });
}

#[cfg(target_arch = "wasm32")]
pub fn file_dialogue_export_image(
    context: egui::Context,
    session: &mut Session,
    ui_state: &mut AppUiState,
) {
    session.editor.commit_transient_edit(false);
    let final_image = ui_state
        .export_image_selected_resolution
        .as_ref()
        .unwrap()
        .clone();
    let final_image = session
        .toolbox
        .convert_color_space(final_image, ColorSpace::sRGB);
    let final_image = session
        .toolbox
        .convert_image_format(final_image, ImageFormat::Rgba8Unorm);
    let mut image_reader = ImageReaderJpeg::new(
        session.runtime.clone(),
        session.toolbox.clone(),
        final_image,
    );

    let output_file_name = ui_state
        .export_file_name
        .clone()
        .expect("expecting an export file name");

    execute(async move {
        let jpeg_data = image_reader.await_jpeg_data().await;
        let array = Uint8Array::from(jpeg_data.as_slice());
        let blob_parts = Array::new();
        blob_parts.push(&array.buffer());

        let file = File::new_with_blob_sequence_and_options(
            &blob_parts.into(),
            output_file_name.as_str(),
            web_sys::FilePropertyBag::new().type_("image/jpeg"),
        )
        .unwrap();
        let url = Url::create_object_url_with_blob(&file);
        if let Some(window) = web_sys::window() {
            let document = window.document().unwrap();
            let body = document.body().unwrap();
            let a = document
                .create_element("a")
                .unwrap()
                .dyn_into::<web_sys::HtmlAnchorElement>()
                .unwrap();
            a.set_href(&url.unwrap());
            a.set_download(output_file_name.as_str());
            body.append_child(&a).unwrap();
            a.click();
            body.remove_child(&a).unwrap();
        }
    });
}

#[cfg(target_arch = "wasm32")]
pub fn file_dialogue_export_edit(
    context: egui::Context,
    session: &mut Session,
    ui_state: &mut AppUiState,
) {
    let edit = session.editor.get_full_size_edit();
    let edit_json_str = serde_json::to_string_pretty(&edit).expect("failed to serialize to json");

    let mut output_file_name = "edit.json".to_owned();
    if let Some(identifier) = session.editor.current_image_identifier() {
        if let Some(name) = session.library.get_metadata(&identifier).name {
            output_file_name = edit_json_file_name(&name);
        }
    }

    execute(async move {
        let array = Uint8Array::from(edit_json_str.as_bytes());
        let blob_parts = Array::new();
        blob_parts.push(&array.buffer());
        let file = File::new_with_blob_sequence_and_options(
            &blob_parts.into(),
            output_file_name.as_str(),
            web_sys::FilePropertyBag::new().type_("text/json"),
        )
        .unwrap();
        let url = Url::create_object_url_with_blob(&file);
        if let Some(window) = web_sys::window() {
            let document = window.document().unwrap();
            let body = document.body().unwrap();
            let a = document
                .create_element("a")
                .unwrap()
                .dyn_into::<web_sys::HtmlAnchorElement>()
                .unwrap();
            a.set_href(&url.unwrap());
            a.set_download(output_file_name.as_str());
            body.append_child(&a).unwrap();
            a.click();
            body.remove_child(&a).unwrap();
        }
    });
}

fn edit_json_file_name(name: &String) -> String {
    let parts: Vec<&str> = name.rsplitn(2, '.').collect();
    if parts.len() == 2 {
        format!("{}_edit.{}", parts[1], "json")
    } else {
        format!("{}_edit.json", name)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || futures::executor::block_on(f));
}

#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use js_sys::{Array, ArrayBuffer, Uint8Array};
#[cfg(target_arch = "wasm32")]
use web_sys::{window, Blob, File, FileReader, HtmlInputElement, Url};

#[cfg(target_arch = "wasm32")]
pub struct ImageImportDialog {
    channel: (
        std::sync::mpsc::Sender<AddedImageOrAlbum>,
        std::sync::mpsc::Receiver<AddedImageOrAlbum>,
    ),
    runtime: Arc<Runtime>,
    toolbox: Arc<Toolbox>,
    context: egui::Context,
    input: HtmlInputElement,
    closure: Option<Closure<dyn FnMut()>>,
}

#[cfg(target_arch = "wasm32")]
impl Drop for ImageImportDialog {
    fn drop(&mut self) {
        self.input.remove();
        if self.closure.is_some() {
            std::mem::replace(&mut self.closure, None).unwrap().forget();
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl ImageImportDialog {
    pub fn new(runtime: Arc<Runtime>, toolbox: Arc<Toolbox>, context: egui::Context) -> Self {
        let document = window().unwrap().document().unwrap();
        let body = document.body().unwrap();
        let input = document
            .create_element("input")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();
        input.set_attribute("type", "file").unwrap();
        input.set_attribute("multiple", "true").unwrap();
        input.style().set_property("display", "none").unwrap();
        body.append_child(&input).unwrap();

        Self {
            channel: std::sync::mpsc::channel(),
            runtime,
            toolbox,
            context,

            input,
            closure: None,
        }
    }

    pub fn open_pick_images(&mut self) {
        if let Some(closure) = &self.closure {
            self.input
                .remove_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
                .unwrap();
            std::mem::replace(&mut self.closure, None).unwrap().forget();
        }

        let runtime = self.runtime.clone();
        let toolbox = self.toolbox.clone();
        let context = self.context.clone();
        let sender = self.channel.0.clone();
        let input_clone = self.input.clone();

        let closure = Closure::once(move || {
            if let Some(files) = input_clone.files() {
                for i in 0..files.length() {
                    let file = files.get(i).unwrap();
                    let file_name = file.name();
                    let file_name_parts: Vec<&str> = file_name.split(".").collect();
                    let ext = file_name_parts.last().unwrap().to_string();

                    let reader = FileReader::new().unwrap();
                    let reader_clone = reader.clone();
                    let runtime = runtime.clone();
                    let toolbox = toolbox.clone();
                    let context = context.clone();
                    let sender = sender.clone();

                    let onload_closure = Closure::once(Box::new(move || {
                        let array_buffer = reader_clone
                            .result()
                            .unwrap()
                            .dyn_into::<ArrayBuffer>()
                            .unwrap();
                        let image_data = Uint8Array::new(&array_buffer).to_vec();
                        let image = runtime.create_image_from_bytes_and_extension(
                            image_data.as_slice(),
                            ext.as_str(),
                        );
                        match image {
                            Ok(image) => {
                                let metadata = LibraryImageMetaData {
                                    name: Some(file_name),
                                };
                                let image = Arc::new(image);
                                let added_img = AddedImageOrAlbum::Image(image, metadata);
                                sender.send(added_img).expect("failed to send added image");
                                context.request_repaint();
                            }
                            Err(_) => {}
                        }
                    }));

                    reader.set_onload(Some(onload_closure.as_ref().unchecked_ref()));
                    reader.read_as_array_buffer(&file).unwrap();
                    onload_closure.forget();
                }
            }
        });

        self.input
            .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
            .unwrap();
        self.closure = Some(closure);
        self.input.click();
    }

    pub fn get_added_image(&mut self) -> Option<AddedImageOrAlbum> {
        if let Ok(added_image) = self.channel.1.try_recv() {
            Some(added_image)
        } else {
            None
        }
    }
}

// native

#[cfg(not(target_arch = "wasm32"))]
pub struct ImageImportDialog {
    channel: (
        std::sync::mpsc::Sender<AddedImageOrAlbum>,
        std::sync::mpsc::Receiver<AddedImageOrAlbum>,
    ),
    runtime: Arc<Runtime>,
    toolbox: Arc<Toolbox>,
    context: egui::Context,
}

#[cfg(not(target_arch = "wasm32"))]
impl ImageImportDialog {
    pub fn new(runtime: Arc<Runtime>, toolbox: Arc<Toolbox>, context: egui::Context) -> Self {
        Self {
            channel: std::sync::mpsc::channel(),
            runtime,
            toolbox,
            context,
        }
    }

    pub fn open_pick_images(&mut self) {
        let task = rfd::AsyncFileDialog::new()
            .add_filter("extension", &["png", "jpg", "jpeg"])
            .pick_files();

        let sender = self.channel.0.clone();
        let runtime = self.runtime.clone();
        let context = self.context.clone();

        execute(async move {
            let files = task.await;
            if let Some(files) = files {
                let mut paths = Vec::new();
                for file in files {
                    let pathbuf = file.path().to_path_buf();
                    paths.push(pathbuf);
                }
                let added_image = AddedImageOrAlbum::ImagesFromPaths(paths);
                sender
                    .send(added_image)
                    .expect("failed to send added image");
                context.request_repaint();
            }
        });
    }

    pub fn open_pick_folder(&mut self) {
        let task = rfd::AsyncFileDialog::new().pick_folder();

        let sender = self.channel.0.clone();
        let runtime = self.runtime.clone();
        let context = self.context.clone();

        execute(async move {
            let file = task.await;
            if let Some(file) = file {
                let added_album = AddedImageOrAlbum::AlbumFromPath(file.path().to_path_buf());
                sender
                    .send(added_album)
                    .expect("failed to send added image");
                context.request_repaint();
            }
        });
    }

    pub fn get_added_image(&mut self) -> Option<AddedImageOrAlbum> {
        if let Ok(added_image) = self.channel.1.try_recv() {
            Some(added_image)
        } else {
            None
        }
    }
}
