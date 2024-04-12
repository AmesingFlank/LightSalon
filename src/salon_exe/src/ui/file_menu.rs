use super::{AddedImage, AppUiState};
use eframe::{
    egui::{self, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::{
    library::LibraryImageIdentifier,
    runtime::{ColorSpace, ImageFormat, ImageReaderJpeg, Runtime, Toolbox},
    session::Session,
};
use std::{future::Future, ops::Add, sync::Arc};

pub fn file_menu(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    ui.menu_button("File", |ui| {
        if ui
            .add_enabled(true, egui::Button::new("Import Image"))
            .clicked()
        {
            ui.close_menu();
            ui_state.import_image_dialog.open();
        }

        let has_current_img = session.editor.current_edit_context_ref().is_some();
        if ui
            .add_enabled(has_current_img, egui::Button::new("Export Image"))
            .clicked()
        {
            ui.close_menu();
            let ctx = ui.ctx().clone();
            file_dialogue_export_image(ctx, session, ui_state);
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn file_dialogue_export_image(
    context: egui::Context,
    session: &mut Session,
    ui_state: &mut AppUiState,
) {
    let task = rfd::AsyncFileDialog::new()
        .add_filter("extension", &["jpg"])
        .save_file();
    let runtime = session.runtime.clone();

    if let Some(context) = session.editor.current_edit_context_mut() {
        let result = session.editor.execute_current_edit_original_size();
        let final_image = result.final_image.clone();
        let final_image = session
            .toolbox
            .convert_color_space(final_image, ColorSpace::sRGB);
        let final_image = session
            .toolbox
            .convert_image_format(final_image, ImageFormat::Rgba8Unorm);
        let mut image_reader =
            ImageReaderJpeg::new(runtime.clone(), session.toolbox.clone(), final_image);
        execute(async move {
            let file = task.await;
            let jpeg_data = image_reader.await_jpeg_data().await;
            if let Some(file) = file {
                file.write(&jpeg_data).await.expect("Write file failed");
            }
        });
    }
}

#[cfg(target_arch = "wasm32")]
fn file_dialogue_export_image(
    context: egui::Context,
    session: &mut Session,
    ui_state: &mut AppUiState,
) {
    let runtime = session.runtime.clone();

    if let Some(context) = session.editor.current_edit_context_mut() {
        let result = session.editor.execute_current_edit_original_size();
        let final_image = result.final_image.clone();
        let final_image = session
            .toolbox
            .convert_color_space(final_image, ColorSpace::sRGB);
        let final_image = session
            .toolbox
            .convert_image_format(final_image, ImageFormat::Rgba8Unorm);
        let mut image_reader =
            ImageReaderJpeg::new(runtime.clone(), session.toolbox.clone(), final_image);
        execute(async move {
            let jpeg_data = image_reader.await_jpeg_data().await;
            let array = Uint8Array::from(jpeg_data.as_slice());
            let blob_parts = Array::new();
            blob_parts.push(&array.buffer());

            let file = File::new_with_blob_sequence_and_options(
                &blob_parts.into(),
                "output.jpg",
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
                a.set_download("output.jpg");
                body.append_child(&a).unwrap();
                a.click();
                body.remove_child(&a).unwrap();
            }
        });
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
        std::sync::mpsc::Sender<AddedImage>,
        std::sync::mpsc::Receiver<AddedImage>,
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

    pub fn open(&mut self) {
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
                                let image = Arc::new(image);
                                let added_img = AddedImage::Image(image);
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

    pub fn get_added_image(&mut self) -> Option<AddedImage> {
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
        std::sync::mpsc::Sender<AddedImage>,
        std::sync::mpsc::Receiver<AddedImage>,
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

    pub fn open(&mut self) {
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
                let added_image = AddedImage::ImagesFromPaths(paths);
                sender
                    .send(added_image)
                    .expect("failed to send added image");
                context.request_repaint();
            }
        });
    }

    pub fn get_added_image(&mut self) -> Option<AddedImage> {
        if let Ok(added_image) = self.channel.1.try_recv() {
            Some(added_image)
        } else {
            None
        }
    }
}
