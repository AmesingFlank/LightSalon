use eframe::{
    egui::{self, CollapsingHeader, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{editor::GlobalEdit, session::Session};

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
                    .color_override(
                        Color32::from_rgb(50, 130, 230),
                        Color32::from_rgb(255, 230, 50),
                    )
                    .text("Temperature"),
            );
            ui.add(
                EditorSlider::new(&mut edit.tint, -100.0..=100.0)
                    .color_override(
                        Color32::from_rgb(65, 230, 25),
                        Color32::from_rgb(150, 0, 230),
                    )
                    .text("Tint"),
            );
            ui.add(EditorSlider::new(&mut edit.vibrance, -100.0..=100.0).text("Vibrance"));
            ui.add(EditorSlider::new(&mut edit.saturation, -100.0..=100.0).text("Saturation"));
        });
}
