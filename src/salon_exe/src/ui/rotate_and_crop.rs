use eframe::{
    egui::{self, CollapsingHeader, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{
    editor::Edit,
    session::Session,
    utils::{
        math::{
            get_cropped_image_dimensions, handle_new_rotation, integer_aspect_ratio,
            maybe_shrink_crop_rect_due_to_rotation,
        },
        rectangle::Rectangle,
    },
};

use super::{widgets::EditorSlider, AppUiState};

pub fn rotate_and_crop(
    ui: &mut Ui,
    session: &mut Session,
    ui_state: &mut AppUiState,
    edit: &mut Edit,
) {
    ui.spacing_mut().slider_width = ui.available_width() * 0.6;

    ui.horizontal(|ui| {
        let input_image = session
            .editor
            .current_edit_context_ref()
            .unwrap()
            .input_image();
        let crop_rect = edit.crop_rect.clone().unwrap_or(Rectangle::regular());
        let output_dimensions =
            get_cropped_image_dimensions(input_image.properties.dimensions, crop_rect);
        let (mut x, mut y) = integer_aspect_ratio(output_dimensions);
        ui.label("Aspect Ratio: ");
        ui.label("Width ");
        ui.add(egui::DragValue::new(&mut x).clamp_range(0..=21));
        ui.label(" x ");
        ui.label("Height ");
        ui.add(egui::DragValue::new(&mut y).clamp_range(0..=21));
    });

    ui.separator();

    let mut rotation_degrees = edit.rotation_degrees.clone().unwrap_or(0.0);
    ui.add(
        EditorSlider::new(&mut rotation_degrees, -180.0..=180.0)
            .double_click_reset_value(0.0)
            .step_by(0.01)
            .fixed_decimals(2)
            .text("Rotation"),
    );
    handle_new_rotation(session, edit, rotation_degrees);
}
