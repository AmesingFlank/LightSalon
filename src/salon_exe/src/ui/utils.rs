use eframe::{
    egui::{self, Ui},
    epaint::Pos2,
};
use salon_core::{
    editor::Edit,
    runtime::Image,
    session::Session,
    utils::vec::{vec2, Vec2},
};

use super::AppUiState;

pub fn pos2_to_vec2(p: Pos2) -> Vec2<f32> {
    vec2((p.x, p.y))
}

pub fn legalize_ui_state(ui_state: &mut AppUiState, edit: &Edit) {
    if ui_state.selected_mask_index >= edit.masked_edits.len() {
        ui_state.selected_mask_index = 0;
        ui_state.selected_mask_term_index = None;
    }
    if let Some(term) = ui_state.selected_mask_term_index {
        if term
            >= edit.masked_edits[ui_state.selected_mask_index]
                .mask
                .terms
                .len()
        {
            ui_state.selected_mask_term_index = None;
        }
    }
}
