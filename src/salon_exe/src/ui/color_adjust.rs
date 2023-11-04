use eframe::{
    egui::{CollapsingHeader, Ui, self},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{editor::GlobalEdit, session::Session};

use super::AppUiState;

pub fn color_adjust(
    ui: &mut Ui,
    session: &mut Session,
    ui_state: &mut AppUiState,
    editor_state: &mut GlobalEdit,
) {
    CollapsingHeader::new("Color")
        .default_open(true)
        .show(ui, |ui| {
            ui.spacing_mut().slider_width = ui.available_width() * 0.6;
            ui.add(
                egui::Slider::new(&mut editor_state.temperature_val, -100.0..=100.0)
                    .text("Temperature"),
            );
            ui.add(egui::Slider::new(&mut editor_state.tint_val, -100.0..=100.0).text("Tint"));
            ui.add(
                egui::Slider::new(&mut editor_state.vibrance_val, -100.0..=100.0).text("Vibrance"),
            );
            ui.add(
                egui::Slider::new(&mut editor_state.saturation_val, -100.0..=100.0)
                    .text("Saturation"),
            );
        });
}
