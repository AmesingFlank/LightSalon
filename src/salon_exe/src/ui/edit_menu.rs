use eframe::{
    egui::{self, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::session::Session;

use super::{utils::legalize_ui_state, AppUiState};

pub fn edit_menu(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    ui.menu_button("Edit", |ui| {
        let can_undo = session.editor.can_undo();
        if ui
            .add_enabled(can_undo, egui::Button::new("Undo"))
            .clicked()
        {
            undo_action(session, ui_state);
        }

        let can_redo = session.editor.can_redo();
        if ui
            .add_enabled(can_redo, egui::Button::new("Redo"))
            .clicked()
        {
            redo_action(session, ui_state);
        }
    });
}

pub fn undo_action(session: &mut Session, ui_state: &mut AppUiState) {
    session.editor.maybe_undo();
    session.update_thumbnail_for_current_image();
    if let Some(context) = session.editor.current_edit_context_ref() {
        legalize_ui_state(ui_state, context.current_edit_ref());
    }
}

pub fn redo_action(session: &mut Session, ui_state: &mut AppUiState) {
    session.editor.maybe_redo();
    session.update_thumbnail_for_current_image();
    if let Some(context) = session.editor.current_edit_context_ref() {
        legalize_ui_state(ui_state, context.current_edit_ref());
    }
}
