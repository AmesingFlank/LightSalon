use eframe::{
    egui::{self, CollapsingHeader, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{
    editor::Edit,
    ir::Frame,
    session::Session,
    utils::{
        math::{
            approximate_aspect_ratio, get_cropped_image_dimensions,
            get_max_crop_rect_with_aspect_ratio, handle_new_crop_rect, handle_new_rotation,
            maybe_shrink_crop_rect_due_to_rotation, reduced_aspect_ratio,
        },
        rectangle::Rectangle,
    },
};

use super::{widgets::EditorSlider, AppUiState};

pub fn framing(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState, edit: &mut Edit) {
    ui.spacing_mut().slider_width = ui.available_width() * 0.6;

    let mut use_framing = edit.framing.is_some();
    ui.checkbox(&mut use_framing, "Add Framing");

    if use_framing {
        if edit.framing.is_none() {
            edit.framing = Some(Frame::defualt())
        }
    } else {
        edit.framing = None;
    }

    if use_framing {
        let mut framing = edit.framing.clone().unwrap();

        ui.horizontal(|ui| {
            let old_aspect_ratio = approximate_aspect_ratio(framing.aspect_ratio, 21);

            let mut aspect_ratio = old_aspect_ratio;

            ui.label("Aspect Ratio ");
            ui.label("Width ");
            let clamp_range = 1..=10000;
            ui.add(egui::DragValue::new(&mut aspect_ratio.0).range(clamp_range.clone()));
            ui.label(" x ");
            ui.label("Height ");
            ui.add(egui::DragValue::new(&mut aspect_ratio.1).range(clamp_range.clone()));
            let new_reduced_aspect_ratio = reduced_aspect_ratio(aspect_ratio);
            if new_reduced_aspect_ratio != reduced_aspect_ratio(old_aspect_ratio) {
                framing.aspect_ratio = new_reduced_aspect_ratio;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Gap ");
            ui.add(
                EditorSlider::new(&mut framing.gap, 0.0..=1.0)
                    .double_click_reset_value(Frame::defualt().gap as f64),
            );
        });

        edit.framing = Some(framing);
    }
}
