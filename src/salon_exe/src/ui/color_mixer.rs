use std::f32::consts::PI;

use eframe::{
    egui::{self, CollapsingHeader, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{
    editor::GlobalEdit,
    runtime::ColorSpace,
    session::Session,
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
                Color32::from_rgb(128, 0, 20), // HSLuv (0.0 / 8.0 * 2 * PI, 100, 40)
                Color32::from_rgb(148, 56, 0), // HSLuv (1.0 / 8.0 * 2 * PI, 100, 60)
                Color32::from_rgb(71, 79, 0),  // HSLuv (2.0 / 8.0 * 2 * PI, 100, 60)
                Color32::from_rgb(0, 98, 18),  // HSLuv (3.0 / 8.0 * 2 * PI, 100, 60)
                Color32::from_rgb(0, 92, 78),  // HSLuv (4.0 / 8.0 * 2 * PI, 100, 60)
                Color32::from_rgb(0, 86, 139), // HSLuv (5.0 / 8.0 * 2 * PI, 100, 60)
                Color32::from_rgb(30, 5, 255), // HSLuv (6.0 / 8.0 * 2 * PI, 100, 40)
                Color32::from_rgb(105, 0, 87), // HSLuv (7.0 / 8.0 * 2 * PI, 100, 40)
            ];

            let colors_checked: [Color32; 8] = [
                Color32::from_rgb(255, 177, 189), // HSLuv (0.0 / 8.0 * 2 * PI, 100, 90)
                Color32::from_rgb(255, 182, 138), // HSLuv (1.0 / 8.0 * 2 * PI, 100, 90)
                Color32::from_rgb(193, 215, 0),  // HSLuv (2.0 / 8.0 * 2 * PI, 100, 90)
                Color32::from_rgb(33, 255, 72),  // HSLuv (3.0 / 8.0 * 2 * PI, 100, 90)
                Color32::from_rgb(0, 251, 212),  // HSLuv (4.0 / 8.0 * 2 * PI, 100, 90)
                Color32::from_rgb(130, 208, 255), // HSLuv (5.0 / 8.0 * 2 * PI, 100, 90)
                Color32::from_rgb(195, 188, 255), // HSLuv (6.0 / 8.0 * 2 * PI, 100, 90)
                Color32::from_rgb(255, 172, 240), // HSLuv (7.0 / 8.0 * 2 * PI, 100, 90)
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
