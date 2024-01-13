use eframe::{
    egui::{self, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::{library::AddImageResult, session::Session};

use super::AppUiState;

pub fn file_menu(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    ui.menu_button("File", |ui| {
        if ui.button("Import Image").clicked() {
            ui.close_menu();
            file_dialogue_import_image(session, ui_state);
        }
    });
}

fn file_dialogue_import_image(session: &mut Session, ui_state: &mut AppUiState) {
    if let Some(path) = rfd::FileDialog::new().pick_file() {
        let add_result = session.library.as_mut().add(path.to_str().unwrap());
        let selected_image: Option<usize> = match add_result {
            AddImageResult::AddedNewImage(i) => Some(i),
            AddImageResult::ImageAlreadyExists(i) => Some(i),
            AddImageResult::Error(_) => None,
        };
        ui_state.reset_for_different_image();
        match selected_image {
            Some(i) => session.set_current_image(i),
            None => {}
        };
    }
}
