use eframe::{
    egui::{self, CollapsingHeader, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{editor::GlobalEdit, runtime::ColorSpace, session::Session};

use super::{widgets::EditorSlider, AppUiState};

pub fn effects(
    ui: &mut Ui,
    session: &mut Session,
    ui_state: &mut AppUiState,
    edit: &mut GlobalEdit,
) {
    CollapsingHeader::new("Effects")
        .default_open(true)
        .show(ui, |ui| {
            ui.spacing_mut().slider_width = ui.available_width() * 0.6;
            ui.add(
                EditorSlider::new(&mut edit.dehaze, 0.0..=100.0)
                    .double_click_reset_value(0.0)
                    .text("Dehaze"),
            );
            ui.add(
                EditorSlider::new(&mut edit.vignette, -100.0..=100.0)
                    .double_click_reset_value(0.0)
                    .text("Vignette"),
            );
        });
}
