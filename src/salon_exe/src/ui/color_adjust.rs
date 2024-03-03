use eframe::{
    egui::{self, CollapsingHeader, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{editor::GlobalEdit, runtime::ColorSpace, session::Session};

use super::{widgets::EditorSlider, AppUiState};

pub fn color_adjust(
    ui: &mut Ui,
    session: &mut Session,
    ui_state: &mut AppUiState,
    edit: &mut GlobalEdit,
) {
    CollapsingHeader::new("Color")
        .default_open(true)
        .show(ui, |ui| {
            ui.spacing_mut().slider_width = ui.available_width() * 0.6;
            ui.add(
                EditorSlider::new(&mut edit.temperature, -100.0..=100.0)
                    .color_override([0.2, 0.5, 0.9], [1.0, 0.9, 0.2], ColorSpace::LinearRGB)
                    .double_click_reset_value(0.0)
                    .text("Temperature"),
            );
            ui.add(
                EditorSlider::new(&mut edit.tint, -100.0..=100.0)
                    .color_override([0.3, 0.9, 0.1], [0.6, 0.0, 0.9], ColorSpace::LinearRGB)
                    .double_click_reset_value(0.0)
                    .text("Tint"),
            );
            ui.add(
                EditorSlider::new(&mut edit.vibrance, -100.0..=100.0)
                    .double_click_reset_value(0.0)
                    .text("Vibrance"),
            );
            ui.add(
                EditorSlider::new(&mut edit.saturation, -100.0..=100.0)
                    .double_click_reset_value(0.0)
                    .text("Saturation"),
            );
        });
}
