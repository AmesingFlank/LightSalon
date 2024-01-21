use eframe::{
    egui::{self, Modifiers, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::{library::AddImageResult, session::Session};

use super::{redo_action, undo_action, utils::legalize_ui_state, AppUiState, EditorPanel};

pub fn keyboard_response(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    ui.input(|i| {
        if i.key_pressed(egui::Key::Enter) {
            if ui_state.editor_panel == EditorPanel::CropAndRotate {
                ui_state.editor_panel = EditorPanel::LightAndColor;
                session.editor.commit_transient_edit(true);
            }
        }
        let control_or_comand = i.modifiers.command || i.modifiers.ctrl || i.modifiers.mac_cmd;
        if i.key_pressed(egui::Key::Z) && control_or_comand {
            if i.modifiers.shift {
                redo_action(session, ui_state)
            } else {
                undo_action(session, ui_state);
            }
        }
    });
}
