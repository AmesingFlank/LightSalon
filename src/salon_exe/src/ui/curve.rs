use eframe::{
    egui::{CollapsingHeader, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{editor::EditorState, session::Session};

use super::AppUiState;

pub fn curve(
    ui: &mut Ui,
    session: &mut Session,
    ui_state: &mut AppUiState,
    editor_state: &mut EditorState,
) {
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

                        let mut control_points_non_highlighted: Vec<[f64; 2]> = Vec::new();
                        let mut highlighted_point_index = None;

                        if let Some(ref selected) = ui_state.selected_curve_control_point_index {
                            highlighted_point_index = Some(*selected);
                        } else {
                            for i in 0..editor_state.curve_control_points.len() {
                                let p = editor_state.curve_control_points[i];
                                if let Some(ref coords) = ptr_coords {
                                    if (p.0 - coords.x as f32).abs() < 0.05 {
                                        highlighted_point_index = Some(i);
                                        break;
                                    }
                                }
                            }
                        }

                        for i in 0..editor_state.curve_control_points.len() {
                            if Some(i) != highlighted_point_index {
                                let p = editor_state.curve_control_points[i];
                                let p = [p.0 as f64, p.1 as f64];
                                control_points_non_highlighted.push(p);
                            }
                        }
                        let control_points_non_highlighted =
                            Points::new(control_points_non_highlighted)
                                .shape(MarkerShape::Circle)
                                .radius(ui_state.last_frame_size.unwrap().1 * 0.003)
                                .filled(false)
                                .color(Color32::from_gray(200));

                        plot_ui.points(control_points_non_highlighted);

                        if let Some(ref i) = highlighted_point_index {
                            let p = editor_state.curve_control_points[*i];
                            let p = [p.0 as f64, p.1 as f64];
                            let control_points_highlighted = Points::new(vec![p])
                                .shape(MarkerShape::Circle)
                                .radius(ui_state.last_frame_size.unwrap().1 * 0.005)
                                .filled(true)
                                .color(Color32::from_gray(255));
                            plot_ui.points(control_points_highlighted);
                        }
                        ptr_coords
                    });
                    
                    if response.response.dragged() || response.response.drag_started() {
                        if ui_state.selected_curve_control_point_index.is_none() {
                            if let Some(ref coords) = response.inner {
                                for i in 0..editor_state.curve_control_points.len() {
                                    let p = editor_state.curve_control_points[i];
                                    if (p.0 - coords.x as f32).abs() < 0.05 {
                                        ui_state.selected_curve_control_point_index = Some(i);
                                        break;
                                    }
                                }
                            }
                        }
                        let delta = response.response.drag_delta().y;
                        let delta = delta * -1.0 / ui.available_height();
                        if let Some(selected) = ui_state.selected_curve_control_point_index.as_ref()
                        {
                            let mut p = editor_state.curve_control_points[*selected];
                            p.1 += delta;
                            p.1 = p.1.min(1.0).max(0.0);
                            editor_state.curve_control_points[*selected] = p;
                        }
                    }

                    if response.response.clicked()
                        || (response.response.drag_started()
                            && ui_state.selected_curve_control_point_index.is_none())
                    {
                        if ui_state.selected_curve_control_point_index.is_none() {
                            if let Some(ref coords) = response.inner {
                                let new_point = (coords.x as f32, coords.y as f32);
                                for i in 0..editor_state.curve_control_points.len() - 1 {
                                    let this_p = editor_state.curve_control_points[i];
                                    let next_p = editor_state.curve_control_points[i + 1];
                                    if this_p.0 < new_point.0 && new_point.0 < next_p.0 {
                                        let new_point_idx = i + 1;
                                        editor_state
                                            .curve_control_points
                                            .insert(new_point_idx, new_point);
                                        if response.response.drag_started() {
                                            ui_state.selected_curve_control_point_index =
                                                Some(new_point_idx);
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    if response.response.drag_released() {
                        ui_state.selected_curve_control_point_index = None;
                    }
                }
            }
        });
}
