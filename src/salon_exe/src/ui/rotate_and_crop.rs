use eframe::{
    egui::{self, CollapsingHeader, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{
    editor::Edit,
    session::Session,
    utils::{math::maybe_shrink_crop_rect_due_to_rotation, rectangle::Rectangle},
};

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
    } else if edit.rotation_degrees != Some(rotation_degrees) {
        edit.rotation_degrees = Some(rotation_degrees);
        let context = session.editor.current_edit_context_ref().unwrap();
        let aspect_ratio = context.input_image().aspect_ratio();
        let current_rect = edit.crop_rect.clone().unwrap_or(Rectangle::regular());
        let new_rect = maybe_shrink_crop_rect_due_to_rotation(rotation_degrees, current_rect, aspect_ratio);
        if let Some(rect) = new_rect {
            edit.crop_rect = Some(rect)
        }
    }
}
