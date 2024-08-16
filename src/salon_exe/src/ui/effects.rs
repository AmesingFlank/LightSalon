use eframe::egui::{CollapsingHeader, Ui};

use salon_core::{editor::GlobalEdit, ir::Vignette, session::Session};

use super::{widgets::EditorSlider, AppUiState};

pub fn effects(
    ui: &mut Ui,
    _session: &mut Session,
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
            let defualt_vignette = Vignette::new();
            ui.horizontal(|ui| {
                ui.add(
                    EditorSlider::new(&mut edit.vignette.vignette, -100.0..=100.0)
                        .double_click_reset_value(defualt_vignette.vignette as f64)
                        .text("Vignette"),
                );
                let expand_icon = if ui_state.vignette_expanded {
                    "⏷"
                } else {
                    "⏴"
                };
                if ui
                    .selectable_label(ui_state.vignette_expanded, expand_icon)
                    .clicked()
                {
                    ui_state.vignette_expanded = !ui_state.vignette_expanded;
                }
            });
            if ui_state.vignette_expanded {
                ui.add(
                    EditorSlider::new(&mut edit.vignette.midpoint, 0.0..=100.0)
                        .double_click_reset_value(defualt_vignette.midpoint as f64)
                        .text("Midpoint"),
                );
                ui.add(
                    EditorSlider::new(&mut edit.vignette.feather, 0.0..=100.0)
                        .double_click_reset_value(defualt_vignette.feather as f64)
                        .text("Feather"),
                );
                ui.add(
                    EditorSlider::new(&mut edit.vignette.roundness, 0.0..=100.0)
                        .double_click_reset_value(defualt_vignette.roundness as f64)
                        .text("Roundness"),
                );
            }
        });
}
