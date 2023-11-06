use eframe::{
    egui::{self, CollapsingHeader, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{editor::GlobalEdit, session::Session};

use super::{widgets::EditorSlider, AppUiState};

pub fn color_mixer(
    ui: &mut Ui,
    session: &mut Session,
    ui_state: &mut AppUiState,
    edit: &mut GlobalEdit,
) {
    CollapsingHeader::new("Color Mixer")
        .default_open(true)
        .show(ui, |ui| {
            let colors: [(f32, f32, f32);8] = [(0.0, 0.0, 0.0); 8];
        });
}
