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
            ui.spacing_mut().slider_width = ui.available_width() * 0.6;

            let mut colors_base: [Color32; 8] = [Color32::BLACK; 8];
            for i in 0..8usize {
                colors_base[i] = color_from_hsl(i as f32 / 8.0, 1.0, 0.3);
            }

            let mut colors_checked: [Color32; 8] = [Color32::BLACK; 8];
            for i in 0..8usize {
                colors_checked[i] = color_from_hsl(i as f32 / 8.0, 1.0, 0.8);
            }

            ui.horizontal(|ui| {
                for i in 0..8usize {
                    let response = ui.add(ColoredRadioButton::new(
                        ui_state.color_mixer_color_index == i,
                        "",
                        colors_base[i],
                        colors_checked[i],
                    ));
                    if response.clicked() {
                        ui_state.color_mixer_color_index = i;
                    };
                    if i != 7 {
                        ui.separator();
                    }
                }
            });

            let index = ui_state.color_mixer_color_index;

            let mut base_hue = index as f32 / 8.0;

            let mut left_hue = (base_hue - 1.0 / 8.0) % 1.0;
            if left_hue < 0.0 {
                left_hue = left_hue + 1.0;
            }
            let right_hue = (base_hue + 1.0 / 8.0) % 1.0;
            let left_hue_color = color_from_hsl(left_hue, 1.0, 0.3);
            let right_hue_color = color_from_hsl(right_hue, 1.0, 0.3);

            ui.add(
                EditorSlider::new(&mut edit.color_mixer_edits[index].hue, -100.0..=100.0)
                    .color_override(left_hue_color, right_hue_color, true)
                    .text("Hue"),
            );

            base_hue = base_hue + edit.color_mixer_edits[index].hue * (1.0 / 100.0) * (1.0 / 8.0);
            if base_hue < 0.0 {
                base_hue = base_hue + 1.0;
            }

            let left_saturation_color = color_from_hsl(base_hue, 0.1, 0.3);
            let right_saturation_color = color_from_hsl(base_hue, 1.0, 0.3);

            ui.add(
                EditorSlider::new(
                    &mut edit.color_mixer_edits[index].saturation,
                    -100.0..=100.0,
                )
                .color_override(left_saturation_color, right_saturation_color, true)
                .text("Saturation"),
            );

            let left_lightness_color = color_from_hsl(base_hue, 1.0, 0.01);
            let right_lightness_color = color_from_hsl(base_hue, 1.0, 0.9);

            ui.add(
                EditorSlider::new(&mut edit.color_mixer_edits[index].lightness, -100.0..=100.0)
                    .color_override(left_lightness_color, right_lightness_color, true)
                    .text("Lightness"),
            );
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
