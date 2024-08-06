use super::{
    file_dialogues::{file_dialogue_export_edit}, AppPage, AppUiState,
};
use eframe::{
    egui::{self, Ui},
};

use salon_core::{
    session::Session,
};


pub fn file_menu(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    ui.menu_button("File", |ui| {
        if ui
            .add_enabled(true, egui::Button::new("Import Image"))
            .clicked()
        {
            ui.close_menu();
            ui_state.import_image_dialog.open_pick_images();
        }

        #[cfg(not(target_arch = "wasm32"))]
        if ui
            .add_enabled(true, egui::Button::new("Import Folder as Album"))
            .clicked()
        {
            ui.close_menu();
            ui_state.import_image_dialog.open_pick_folder();
        }

        let has_current_img = session.editor.current_edit_context_ref().is_some();
        if ui
            .add_enabled(has_current_img, egui::Button::new("Export Editted Image"))
            .clicked()
        {
            ui.close_menu();
            session.editor.commit_transient_edit(false);
            ui_state.app_page = AppPage::Export;
        }

        if ui
            .add_enabled(has_current_img, egui::Button::new("Export Edit JSON"))
            .clicked()
        {
            ui.close_menu();
            file_dialogue_export_edit(session, ui_state);
        }
    });
}
