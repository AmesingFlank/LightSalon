use eframe::{
    egui::{self, Ui},
    epaint::Pos2,
};
use salon_core::{
    editor::Edit,
    runtime::Image,
    session::Session,
    utils::{
        rectangle::Rectangle,
        vec::{vec2, Vec2},
    },
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

pub fn get_abs_x_in_rect(rect: egui::Rect, relative_x: f32) -> f32 {
    rect.min.x + rect.width() * relative_x
}

pub fn get_abs_y_in_rect(rect: egui::Rect, relative_y: f32) -> f32 {
    rect.min.y + rect.height() * relative_y
}
