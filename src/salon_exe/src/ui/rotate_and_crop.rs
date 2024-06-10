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
            approximate_aspect_ratio, get_cropped_image_dimensions, get_max_crop_rect_with_aspect_ratio, handle_new_crop_rect, handle_new_rotation, maybe_shrink_crop_rect_due_to_rotation, reduced_aspect_ratio
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
        let old_aspect_ratio = approximate_aspect_ratio(output_dimensions, 21);

        if reduced_aspect_ratio(old_aspect_ratio)
            != reduced_aspect_ratio(ui_state.crop_rect_aspect_ratio)
        {
            ui_state.crop_rect_aspect_ratio = old_aspect_ratio;
        }

        ui.label("Aspect Ratio: ");
        ui.label("Width ");
        let clamp_range = 1..=10000;
        ui.add(
            egui::DragValue::new(&mut ui_state.crop_rect_aspect_ratio.0)
                .clamp_range(clamp_range.clone()),
        );
        ui.label(" x ");
        ui.label("Height ");
        ui.add(
            egui::DragValue::new(&mut ui_state.crop_rect_aspect_ratio.1)
                .clamp_range(clamp_range.clone()),
        );
        if reduced_aspect_ratio(old_aspect_ratio)
            != reduced_aspect_ratio(ui_state.crop_rect_aspect_ratio)
        {
            let rotation_degrees = edit.rotation_degrees.clone().unwrap_or(0.0);
            let aspect_ratio =
                ui_state.crop_rect_aspect_ratio.0 as f32 / ui_state.crop_rect_aspect_ratio.1 as f32;
            let new_crop_rect = get_max_crop_rect_with_aspect_ratio(
                rotation_degrees,
                crop_rect,
                input_image.aspect_ratio(),
                aspect_ratio,
            );
            if new_crop_rect != crop_rect {
                handle_new_crop_rect(input_image.aspect_ratio(), edit, new_crop_rect);
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
