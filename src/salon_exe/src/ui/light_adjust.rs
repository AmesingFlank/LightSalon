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
    edit: &mut GlobalEdit,
) {
    CollapsingHeader::new("Light")
        .default_open(true)
        .show(ui, |ui| {
            ui.spacing_mut().slider_width = ui.available_width() * 0.6;
            ui.add(
                EditorSlider::new(&mut edit.exposure, -4.0..=4.0)
                    .double_click_reset_value(0.0)
                    .text("Exposure"),
            );

            ui.add(
                EditorSlider::new(&mut edit.contrast, -100.0..=100.0)
                    .double_click_reset_value(0.0)
                    .text("Contrast"),
            );

            ui.add(
                EditorSlider::new(&mut edit.highlights, -100.0..=100.0)
                    .double_click_reset_value(0.0)
                    .text("Highlights"),
            );

            ui.add(
                EditorSlider::new(&mut edit.shadows, -100.0..=100.0)
                    .double_click_reset_value(0.0)
                    .text("Shadows"),
            );
        });
}
