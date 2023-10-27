use eframe::{
    egui::{CollapsingHeader, Ui},
    epaint::Color32,
};
use egui_plot::{Line, Plot, MarkerShape, Points};
use salon_core::{session::Session, editor::EditorState};

use super::AppUiState;

pub fn curve(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState, editor_state: &mut EditorState) {
    CollapsingHeader::new("Curve")
        .default_open(true)
        .show(ui, |ui| {
            if let Some(ref result) = session.current_process_result {
                if let Some(ref stats) = result.statistics {
                    let margin = 0.02;
                    let plot = Plot::new("curve")
                        .data_aspect(1.0)
                        .view_aspect(1.0)
                        .show_x(false)
                        .show_y(false)
                        .allow_zoom(false)
                        .allow_scroll(false)
                        .allow_double_click_reset(false)
                        .allow_drag(false)
                        .include_x(0.0 - margin)
                        .include_x(1.0 + margin)
                        .include_y(0.0 - margin)
                        .include_y(1.0 + margin)
                        .show_axes([false, false])
                        .show_grid([false, false]);

                    let response = plot.show(ui, |plot_ui| {
                        let ptr_coords = plot_ui.pointer_coordinate();

                        let mut control_points_non_selected: Vec<[f64; 2]> = Vec::new();

                        for i in 0..editor_state.curve_control_points.len() {
                            let p = editor_state.curve_control_points[i];
                            let p = [p.0 as f64, p.1 as f64];
                            let mut selected = false;
                            if let Some(existing_selection) =
                                ui_state.selected_curve_control_point_index.as_ref()
                            {
                                selected = *existing_selection == i;
                            } else {
                                if let Some(coords) = ptr_coords.as_ref() {
                                    if (p[0] - coords.x).abs() < 0.05 {
                                        selected = true;
                                        ui_state.selected_curve_control_point_index = Some(i);
                                    }
                                }
                            }
                            if !selected {
                                control_points_non_selected.push(p);
                            }
                        }
                        let control_points_non_selected = Points::new(control_points_non_selected)
                            .shape(MarkerShape::Circle)
                            .radius(ui_state.last_frame_size.unwrap().1 * 0.003)
                            .filled(false)
                            .color(Color32::from_gray(200));

                        plot_ui.points(control_points_non_selected);

                        if let Some(selected) =
                            ui_state.selected_curve_control_point_index.as_ref()
                        {
                            let p = editor_state.curve_control_points[*selected];
                            let p = [p.0 as f64, p.1 as f64];
                            let control_points_selected = Points::new(vec![p])
                                .shape(MarkerShape::Circle)
                                .radius(ui_state.last_frame_size.unwrap().1 * 0.003)
                                .filled(true)
                                .color(Color32::from_gray(255));
                            plot_ui.points(control_points_selected);
                        }
                        ptr_coords
                    });
                    if response.response.clicked() {
                        if let Some(coords) = response.inner {
                            println!("clicked {:?}", coords);
                        }
                    }
                    if response.response.dragged() {
                        let delta = response.response.drag_delta().y;
                        let delta = delta * -1.0 / ui.available_height();
                        if let Some(selected) =
                            ui_state.selected_curve_control_point_index.as_ref()
                        {
                            println!("{:?}", delta);
                            editor_state.curve_control_points[*selected].1 += delta;
                        }
                    }
                }
            }
        });
}
