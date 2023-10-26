use eframe::{egui::{Ui, self}, egui_wgpu};
use egui_extras::{Column, TableBuilder};
use salon_core::{session::Session, library::AddImageResult};

pub fn file_menu(ui: &mut Ui, session: &mut Session) {
    ui.menu_button("File", |ui| {
        if ui.button("Import Image").clicked() {
            ui.close_menu();
            file_dialogue_import_image(session);
        }
    });
}

fn file_dialogue_import_image(session: &mut Session) {
    if let Some(path) = rfd::FileDialog::new().pick_file() {
        let add_result = session.library.as_mut().add(path.to_str().unwrap());
        let selected_image: Option<usize> = match add_result {
            AddImageResult::AddedNewImage(i) => Some(i),
            AddImageResult::ImageAlreadyExists(i) => Some(i),
            AddImageResult::Error(_) => None,
        };
        match selected_image {
            Some(i) => session.set_current_image(i),
            None => {}
        };
    }
}