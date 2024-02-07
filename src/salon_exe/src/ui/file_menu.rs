use super::AppUiState;
use eframe::{
    egui::{self, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::{runtime::ColorSpace, session::Session};
use std::{future::Future, sync::Arc};

pub fn file_menu(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    ui.menu_button("File", |ui| {
        if ui.button("Import Image").clicked() {
            ui.close_menu();
            file_dialogue_import_image(session, ui_state);
        }
    });
}

fn file_dialogue_import_image(session: &mut Session, ui_state: &mut AppUiState) {
    // let task = rfd::AsyncFileDialog::new().pick_file();
    // let runtime = session.runtime.clone();
    // execute(async move {
    //     let file = task.await;
    //     if let Some(file) = file {
    //         if let Some(ext) = file.path().extension() {
    //             let ext = ext.to_str().unwrap_or("");
    //             let image_data = file.read().await;
    //             let image = runtime.create_image_from_bytes_and_extension(image_data.as_slice(), ext);
    //             match image {
    //                 Ok(i) => {
    //                     file.path().
    //                 },
    //                 Err(_) => {},
    //             }
    //         }

    //         let extension = file.path().extension();
    //         let _ = sender.send(String::from_utf8_lossy(&text).to_string());
    //         ctx.request_repaint();
    //     }
    // });
    if let Some(path) = rfd::FileDialog::new().pick_file() {
        let img = session.runtime.create_image_from_path(&path).unwrap();
        let index = session.library.add_image(Arc::new(img));
        ui_state.reset_for_different_image();
        session.set_current_image(index);
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
