use eframe::{
    egui::{self, CollapsingHeader, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{editor::Edit, session::Session};

use super::{widgets::EditorSlider, AppUiState};

pub fn rotate_and_crop(
    ui: &mut Ui,
    session: &mut Session,
    ui_state: &mut AppUiState,
    edit: &mut Edit,
) {
    ui.spacing_mut().slider_width = ui.available_width() * 0.6;
    let mut rotation_degrees = edit.rotation_degrees.clone().unwrap_or(0.0);
    ui.add(
        EditorSlider::new(&mut rotation_degrees, -180.0..=180.0)
            .double_click_reset_value(0.0)
            .step_by(0.01)
            .fixed_decimals(2)
            .text("Rotation"),
    );
    if rotation_degrees == 0.0 {
        edit.rotation_degrees = None
    } else {
        edit.rotation_degrees = Some(rotation_degrees)
    }
}
