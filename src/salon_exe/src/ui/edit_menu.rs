use eframe::{
    egui::{self, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::{library::AddImageResult, session::Session};

use super::AppUiState;

pub fn edit_menu(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    ui.menu_button("Edit", |ui| {
        let can_undo = session.editor.can_undo();
        if ui
            .add_enabled(can_undo, egui::Button::new("Undo"))
            .clicked()
        {
            session.editor.maybe_undo();
        }

        let can_redo = session.editor.can_redo();
        if ui
            .add_enabled(can_redo, egui::Button::new("Redo"))
            .clicked()
        {
            session.editor.maybe_redo();
        }
    });
}
