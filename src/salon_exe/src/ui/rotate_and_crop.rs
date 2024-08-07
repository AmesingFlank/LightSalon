use eframe::{
    egui::{self, Ui},
};

use salon_core::{
    editor::Edit,
    runtime::Runtime,
    session::Session,
    utils::{
        math::{
            approximate_aspect_ratio, get_cropped_image_dimensions,
            get_max_crop_rect_with_aspect_ratio, handle_new_crop_rect, handle_new_rotation, reduced_aspect_ratio,
        },
        rectangle::Rectangle,
    },
};

use super::{widgets::EditorSlider, AppUiState};

pub fn rotate_and_crop(
    ui: &mut Ui,
    session: &mut Session,
    _ui_state: &mut AppUiState,
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

        let mut aspect_ratio = old_aspect_ratio.clone();

        ui.label("Aspect Ratio ");
        ui.label("Width ");
        let clamp_range = 1..=Runtime::get_required_max_texture_dim_1d_2d();
        ui.add(egui::DragValue::new(&mut aspect_ratio.0).range(clamp_range.clone()));
        ui.label(" x ");
        ui.label("Height ");
        ui.add(egui::DragValue::new(&mut aspect_ratio.1).range(clamp_range.clone()));
        if reduced_aspect_ratio(old_aspect_ratio) != reduced_aspect_ratio(aspect_ratio) {
            let rotation_degrees = edit.rotation_degrees.clone().unwrap_or(0.0);
            let aspect_ratio = aspect_ratio.0 as f32 / aspect_ratio.1 as f32;
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

    let mut rotation_degrees = edit.rotation_degrees.clone().unwrap_or(0.0);

    ui.horizontal(|ui| {
        ui.label("Rotation ");
        ui.add(
            EditorSlider::new(&mut rotation_degrees, -180.0..=180.0)
                .double_click_reset_value(0.0)
                .step_by(0.01)
                .fixed_decimals(2),
        );
    });

    handle_new_rotation(input_image.aspect_ratio(), edit, rotation_degrees);
}
