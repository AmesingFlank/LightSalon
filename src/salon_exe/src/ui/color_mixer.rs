use eframe::{
    egui::{self, CollapsingHeader, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{
    editor::GlobalEdit,
    session::Session,
    utils::{color_spaces::hsl_to_rgb, vec::vec3},
};

use super::{
    widgets::{ColoredRadioButton, EditorSlider},
    AppUiState,
};

pub fn color_mixer(
    ui: &mut Ui,
    session: &mut Session,
    ui_state: &mut AppUiState,
    edit: &mut GlobalEdit,
) {
    CollapsingHeader::new("Color Mixer")
        .default_open(true)
        .show(ui, |ui| {
            let mut colors_base: [Color32; 8] = [Color32::BLACK; 8];
            for i in 0..8usize {
                colors_base[i] = color_from_hsl(i as f32 / 8.0, 1.0, 0.3);
            }

            let mut colors_checked: [Color32; 8] = [Color32::BLACK; 8];
            for i in 0..8usize {
                colors_checked[i] = color_from_hsl(i as f32 / 8.0, 1.0, 0.6);
            }

            ui.horizontal(|ui| {
                for i in 0..8usize {
                    let response = ui.add(ColoredRadioButton::new(
                        ui_state.color_mixer_color_index == i,
                        "",
                        colors_base[i],
                        Color32::WHITE,
                    ));
                    if response.clicked() {
                        ui_state.color_mixer_color_index = i;
                    };
                    if i != 7 {
                        ui.separator();
                    }
                }
            });
        });
}

fn color_from_hsl(h: f32, s: f32, l: f32) -> Color32 {
    let hsl = vec3((h, s, l));
    let rgb = hsl_to_rgb(hsl);
    Color32::from_rgb(
        (rgb.x * 255.0) as u8,
        (rgb.y * 255.0) as u8,
        (rgb.z * 255.0) as u8,
    )
}
