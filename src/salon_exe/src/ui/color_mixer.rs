use std::f32::consts::PI;

use eframe::{
    egui::{self, CollapsingHeader, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{
    editor::GlobalEdit,
    image::ColorSpace,
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

            let colors_base: [Color32; 8] = [
                Color32::from_rgb(165, 13, 37),  // LCh (50, 100, 0.0 / 8.0 * 2 * PI)
                Color32::from_rgb(160, 54, 0),   // LCh (60, 100, 1.0 / 8.0 * 2 * PI)
                Color32::from_rgb(71, 82, 0),    // LCh (60, 100, 2.0 / 8.0 * 2 * PI)
                Color32::from_rgb(0, 105, 4),    // LCh (60, 100, 3.0 / 8.0 * 2 * PI)
                Color32::from_rgb(0, 115, 85),   // LCh (60, 100, 4.0 / 8.0 * 2 * PI)
                Color32::from_rgb(0, 98, 195),   // LCh (60, 100, 5.0 / 8.0 * 2 * PI)
                Color32::from_rgb(73, 54, 247),  // LCh (60, 100, 6.0 / 8.0 * 2 * PI)
                Color32::from_rgb(204, 22, 172), // LCh (60, 100, 7.0 / 8.0 * 2 * PI)
            ];

            let colors_checked: [Color32; 8] = [
                Color32::from_rgb(255, 162, 227), // LCh (100, 100, 0.0 / 8.0 * 2 * PI)
                Color32::from_rgb(255, 214, 65),  // LCh (100, 100, 1.0 / 8.0 * 2 * PI)
                Color32::from_rgb(253, 255, 22),  // LCh (100, 100, 2.0 / 8.0 * 2 * PI)
                Color32::from_rgb(50, 255, 101),  // LCh (100, 100, 3.0 / 8.0 * 2 * PI)
                Color32::from_rgb(0, 255, 255),   // LCh (100, 100, 4.0 / 8.0 * 2 * PI)
                Color32::from_rgb(0, 255, 255),   // LCh (100, 100, 5.0 / 8.0 * 2 * PI)
                Color32::from_rgb(255, 222, 255), // LCh (100, 100, 6.0 / 8.0 * 2 * PI)
                Color32::from_rgb(255, 158, 255), // LCh (60, 100, 7.0 / 8.0 * 2 * PI)
            ];

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

            let hue_range = PI * 2.0;
            let group_hue_range = hue_range / 8.0;

            let mut base_hue = index as f32 * group_hue_range;

            let mut left_hue = base_hue - group_hue_range;
            if left_hue < 0.0 {
                left_hue = left_hue + hue_range;
            }
            let mut right_hue = base_hue + group_hue_range;
            if right_hue > hue_range {
                right_hue = right_hue - hue_range;
            }

            ui.add(
                EditorSlider::new(&mut edit.color_mixer_edits[index].hue, -100.0..=100.0)
                    .color_override(
                        [left_hue, 100.0, 60.0],
                        [right_hue, 100.0, 60.0],
                        ColorSpace::HSLuv,
                    )
                    .text("Hue"),
            );

            ui.add(
                EditorSlider::new(
                    &mut edit.color_mixer_edits[index].saturation,
                    -100.0..=100.0,
                )
                .color_override(
                    [base_hue, 0.0, 60.0],
                    [base_hue, 100.0, 60.0],
                    ColorSpace::HSLuv,
                )
                .text("Saturation"),
            );

            ui.add(
                EditorSlider::new(&mut edit.color_mixer_edits[index].lightness, -100.0..=100.0)
                    .color_override(
                        [base_hue, 100.0, 0.0],
                        [base_hue, 100.0, 100.0],
                        ColorSpace::HSLuv,
                    )
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
