use eframe::{
    egui::{self, CollapsingHeader, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{editor::GlobalEdit, session::Session, utils::spline::EvaluatedSpline};

use super::{AppUiState, CurveScope, widgets::ColoredRadioButton};

pub fn curve(
    ui: &mut Ui,
    session: &mut Session,
    ui_state: &mut AppUiState,
    edit: &mut GlobalEdit,
) {
    CollapsingHeader::new("Curve")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let scopes = [CurveScope::RGB, CurveScope::R, CurveScope::G, CurveScope::B];
                let base_colors = [
                    Color32::from_rgb(100, 100, 100),
                    Color32::from_rgb(100, 20, 20),
                    Color32::from_rgb(20, 100, 20),
                    Color32::from_rgb(20, 20, 128),
                ];
                let checked_colors = [
                    Color32::from_rgb(200, 200, 200),
                    Color32::from_rgb(250, 20, 20),
                    Color32::from_rgb(20, 250, 20),
                    Color32::from_rgb(50, 80, 255),
                ];
                for i in 0..4usize  {
                    let scope = scopes[i];
                    let response = ui.add(ColoredRadioButton::new(
                        ui_state.curve_scope == scope,
                        scope.to_string(),
                        base_colors[i],
                        checked_colors[i]
                    ));
                    if response.clicked() {
                        ui_state.curve_scope = scope;
                    };
                    if scope != CurveScope::B {
                        ui.separator();
                    }
                }
            });

            let control_points = match ui_state.curve_scope {
                CurveScope::RGB => &mut edit.curve_control_points_all,
                CurveScope::R => &mut edit.curve_control_points_r,
                CurveScope::G => &mut edit.curve_control_points_g,
                CurveScope::B => &mut edit.curve_control_points_b,
            };

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
                    for i in 0..control_points.len() {
                        let p = control_points[i];
                        if let Some(ref coords) = ptr_coords {
                            if (p.0 - coords.x as f32).abs() < 0.05 {
                                highlighted_point_index = Some(i);
                                break;
                            }
                        }
                    }
                }

                for i in 0..control_points.len() {
                    if Some(i) != highlighted_point_index {
                        let p = control_points[i];
                        let p = [p.0 as f64, p.1 as f64];
                        control_points_non_highlighted.push(p);
                    }
                }
                let control_points_non_highlighted = Points::new(control_points_non_highlighted)
                    .shape(MarkerShape::Circle)
                    .radius(ui_state.last_frame_size.unwrap().1 * 0.003)
                    .filled(false)
                    .color(Color32::from_gray(200));

                plot_ui.points(control_points_non_highlighted);

                if let Some(ref i) = highlighted_point_index {
                    let p = control_points[*i];
                    let p = [p.0 as f64, p.1 as f64];
                    let control_points_highlighted = Points::new(vec![p])
                        .shape(MarkerShape::Circle)
                        .radius(ui_state.last_frame_size.unwrap().1 * 0.005)
                        .filled(true)
                        .color(Color32::from_gray(255));
                    plot_ui.points(control_points_highlighted);
                }

                let evaluated = EvaluatedSpline::from_control_points(&control_points, 1.0, 100);
                let mut curve = Vec::with_capacity(evaluated.y_vals.len());

                for i in 0..evaluated.y_vals.len() {
                    let x = i as f64 / (evaluated.y_vals.len() - 1) as f64;
                    curve.push([x, evaluated.y_vals[i] as f64]);
                }
                let curve = Line::new(curve).color(Color32::from_rgb(200, 200, 200));
                plot_ui.line(curve);
                ptr_coords
            });

            if response.response.dragged() || response.response.drag_started() {
                if ui_state.selected_curve_control_point_index.is_none() {
                    if let Some(ref coords) = response.inner {
                        for i in 0..control_points.len() {
                            let p = control_points[i];
                            if (p.0 - coords.x as f32).abs() < 0.05 {
                                ui_state.selected_curve_control_point_index = Some(i);
                                break;
                            }
                        }
                    }
                }
                let mut dx = response.response.drag_delta().x;
                let mut dy = response.response.drag_delta().y;
                dx = dx * 1.0 / ui.available_height();
                dy = dy * -1.0 / ui.available_height();
                if let Some(ref selected) = ui_state.selected_curve_control_point_index {
                    let mut p = control_points[*selected];

                    p.1 += dy;
                    p.1 = p.1.min(1.0).max(0.0);

                    p.0 += dx;
                    p.0 = p.0.min(1.0).max(0.0);
                    if *selected > 0 {
                        let prev = control_points[*selected - 1];
                        p.0 = p.0.max(prev.0 + 0.05);
                    }
                    if *selected < control_points.len() - 1 {
                        let next = control_points[*selected + 1];
                        p.0 = p.0.min(next.0 - 0.05);
                    }

                    control_points[*selected] = p;
                }
            }

            if response.response.clicked()
                || (response.response.drag_started()
                    && ui_state.selected_curve_control_point_index.is_none())
            {
                if ui_state.selected_curve_control_point_index.is_none() {
                    if let Some(ref coords) = response.inner {
                        let new_point = (coords.x as f32, coords.y as f32);
                        for i in 0..control_points.len() - 1 {
                            let this_p = control_points[i];
                            let next_p = control_points[i + 1];
                            if this_p.0 < new_point.0 && new_point.0 < next_p.0 {
                                let new_point_idx = i + 1;
                                control_points.insert(new_point_idx, new_point);
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
        });
}
