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
            get_cropped_image_dimensions, get_max_crop_rect_with_aspect_ratio,
            handle_new_crop_rect, handle_new_rotation, integer_aspect_ratio,
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

    let input_image = session
        .editor
        .current_edit_context_ref()
        .unwrap()
        .input_image();

    ui.horizontal(|ui| {
        let crop_rect = edit.crop_rect.clone().unwrap_or(Rectangle::regular());
        let output_dimensions =
            get_cropped_image_dimensions(input_image.properties.dimensions, crop_rect);
        let old_aspect_ratio = integer_aspect_ratio(output_dimensions);
        let mut new_aspect_ratio = old_aspect_ratio.clone();
        let rotation_degrees = edit.rotation_degrees.clone().unwrap_or(0.0);
        ui.label("Aspect Ratio: ");
        ui.label("Width ");
        ui.add(egui::DragValue::new(&mut new_aspect_ratio.0).clamp_range(0..=21));
        ui.label(" x ");
        ui.label("Height ");
        ui.add(egui::DragValue::new(&mut new_aspect_ratio.1).clamp_range(0..=21));
        if new_aspect_ratio != old_aspect_ratio {
            let new_crop_rect = get_max_crop_rect_with_aspect_ratio(
                rotation_degrees,
                crop_rect,
                input_image.aspect_ratio(),
                new_aspect_ratio.1 as f32 / new_aspect_ratio.0 as f32,
            );
            if new_crop_rect != crop_rect {
                //handle_new_crop_rect(input_image.aspect_ratio(), edit, new_crop_rect);
            }
        }
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
    handle_new_rotation(input_image.aspect_ratio(), edit, rotation_degrees);
}
