use eframe::{
    egui::{self, CollapsingHeader, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{editor::GlobalEdit, session::Session};

use super::{widgets::EditorSlider, AppUiState};

pub fn light_adjust(
    ui: &mut Ui,
    session: &mut Session,
    ui_state: &mut AppUiState,
    editor_state: &mut GlobalEdit,
) {
    CollapsingHeader::new("Light")
        .default_open(true)
        .show(ui, |ui| {
            ui.spacing_mut().slider_width = ui.available_width() * 0.6;
            ui.add(EditorSlider::new(&mut editor_state.exposure_val, -4.0..=4.0).text("Exposure"));

            ui.add(
                EditorSlider::new(&mut editor_state.contrast_val, -100.0..=100.0).text("Contrast"),
            );

            ui.add(
                EditorSlider::new(&mut editor_state.highlights_val, -100.0..=100.0)
                    .text("Highlights"),
            );

            ui.add(
                EditorSlider::new(&mut editor_state.shadows_val, -100.0..=100.0).text("Shadows"),
            );
        });
}
