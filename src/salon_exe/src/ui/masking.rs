use eframe::egui::{self, CollapsingHeader, Ui};
use salon_core::session::Session;

use super::AppUiState;

pub fn masking(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    CollapsingHeader::new("Masking")
        .default_open(true)
        .show(ui, |ui| {
            ui.spacing_mut().slider_width = ui.available_width() * 0.6;
        });
}
