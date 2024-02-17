use super::{AddedImage, AppUiState};
use eframe::{
    egui::{self, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::{
    runtime::{ColorSpace, ImageFormat, ImageReaderJpeg},
    session::Session,
};
use std::{future::Future, sync::Arc};

pub fn file_menu(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    ui.menu_button("File", |ui| {
        if ui.button("Import Image").clicked() {
            ui.close_menu();
            // Context is wrapped in an Arc so it's cheap to clone as per:
            // > Context is cheap to clone, and any clones refers to the same mutable data (Context uses refcounting internally).
            // Taken from https://docs.rs/egui/0.24.1/egui/struct.Context.html
            let ctx = ui.ctx().clone();
            file_dialogue_import_image(ctx, session, ui_state);
        }

        if ui.button("Export Image").clicked() {
            ui.close_menu();
            let ctx = ui.ctx().clone();
            file_dialogue_export_image(ctx, session, ui_state);
        }
    });
}

fn file_dialogue_import_image(
    context: egui::Context,
    session: &mut Session,
    ui_state: &mut AppUiState,
) {
    let task = rfd::AsyncFileDialog::new()
        .add_filter("extension", &["png", "jpg", "jpeg"])
        .pick_file();
    let runtime = session.runtime.clone();
    let sender = ui_state.added_image_channel.0.clone();

    execute(async move {
        let file = task.await;
        if let Some(file) = file {
            let file_name = file.file_name();
            let file_name_parts: Vec<&str> = file_name.split(".").collect();
            let ext = file_name_parts.last().unwrap().to_owned();

            let image_data = file.read().await;
            let image = runtime.create_image_from_bytes_and_extension(image_data.as_slice(), ext);
            match image {
                Ok(img) => {
                    let added_img = AddedImage {
                        image: Arc::new(img),
                    };
                    let _ = sender.send(added_img);
                    context.request_repaint();
                }
                Err(_) => {}
            }
        }
    });
}

fn file_dialogue_export_image(
    context: egui::Context,
    session: &mut Session,
    ui_state: &mut AppUiState,
) {
    let task = rfd::AsyncFileDialog::new()
        .add_filter("extension", &["jpg"])
        .save_file();
    let runtime = session.runtime.clone();

    if let Some(ref input_img) = session.editor.current_input_image {
        let result = session
            .editor
            .execute_current_edit_original_scale(input_img.clone());
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

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || futures::executor::block_on(f));
}

#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
