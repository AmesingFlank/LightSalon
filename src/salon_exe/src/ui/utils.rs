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

pub fn get_image_size_in_ui(ui: &Ui, image: &Image) -> egui::Vec2 {
    let max_x = ui.available_width();
    let max_y = ui.available_height();
    let ui_aspect_ratio = max_y / max_x;

    let image_aspect_ratio = image.aspect_ratio();

    let size = if image_aspect_ratio >= ui_aspect_ratio {
        egui::Vec2 {
            x: max_y / image_aspect_ratio,
            y: max_y,
        }
    } else {
        egui::Vec2 {
            x: max_x,
            y: max_x * image_aspect_ratio,
        }
    };
    size
}

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
