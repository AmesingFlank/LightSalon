use std::f32::consts::PI;
use std::mem::size_of;
use std::sync::Arc;
use std::{collections::HashMap, num::NonZeroU64};

use eframe::egui::{CursorIcon, Ui};
use eframe::epaint::{Color32, Pos2, Stroke};
use eframe::{egui, egui_wgpu};
use salon_core::editor::Edit;
use salon_core::ir::{LinearGradientMask, MaskPrimitive, RadialGradientMask};
use salon_core::runtime::Image;
use salon_core::runtime::Sampler;
use salon_core::runtime::{
    BindGroupDescriptor, BindGroupDescriptorKey, BindGroupEntry, BindGroupManager, BindingResource,
    Runtime,
};
use salon_core::runtime::{Buffer, BufferProperties, RingBuffer};
use salon_core::session::Session;
use salon_core::shader::{Shader, ShaderLibraryModule};
use salon_core::utils::math::{
    get_crop_rect_translation_bounds, get_crop_rect_upscale_bounds, get_rotation_mat,
    handle_new_crop_rect,
};
use salon_core::utils::rectangle::Rectangle;
use salon_core::utils::vec::{vec2, Vec2};

use super::utils::{get_abs_x_in_rect, get_abs_y_in_rect, get_max_image_size, pos2_to_vec2};
use super::widgets::{ImageGeometryEditCallback, MainImageCallback};
use super::{AppUiState, CropDragEdgeOrCorner, EditorPanel, MaskEditState};

pub fn main_image(
    ctx: &egui::Context,
    ui: &mut Ui,
    session: &mut Session,
    ui_state: &mut AppUiState,
) {
    if session.editor.current_edit_context_ref().is_none() {
        return;
    }
    if ui_state.show_comparison {
        ui.columns(2, |columns| {
            columns[0].centered_and_justified(|ui| {
                let context = session.editor.current_edit_context_ref().unwrap();
                let original_image = context.input_image();

                let main_image_callback = MainImageCallback {
                    image: original_image.clone(),
                    mask: None,
                    ui_max_rect: ui.max_rect(),
                };

                let main_image_rect = main_image_callback.image_ui_rect();

                let _response = ui.allocate_rect(main_image_rect, egui::Sense::drag());

                ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                    main_image_rect,
                    main_image_callback,
                ));
            });
            show_main_image(ctx, &mut columns[1], session, ui_state);
        });
    } else {
        show_main_image(ctx, ui, session, ui_state);
    }
}

fn show_main_image(
    ctx: &egui::Context,
    ui: &mut Ui,
    session: &mut Session,
    ui_state: &mut AppUiState,
) {
    if ui_state.editor_panel == EditorPanel::CropAndRotate {
        image_crop_and_rotate(ctx, ui, session, ui_state);
    } else {
        show_edited_image(ctx, ui, session, ui_state);
    }
}

fn show_edited_image(
    ctx: &egui::Context,
    ui: &mut Ui,
    session: &mut Session,
    ui_state: &mut AppUiState,
) {
    ui.centered_and_justified(|ui| {
        // request to resize the image into a smaller image before applying all other edits, for better perf.
        let context = session.editor.current_edit_context_mut().unwrap();
        let input_image = context.input_image();
        let original_dimensions = input_image.properties.dimensions;
        let mut original_size = vec2((original_dimensions.0 as f32, original_dimensions.1 as f32));
        if let Some(ref crop_rect) = context.current_edit_ref().crop_rect {
            original_size = original_size * crop_rect.size;
        }
        let aspect_ratio = original_size.y / original_size.x;

        let size_in_ui = get_image_size_in_ui(ui, aspect_ratio);
        // HiDPI (aka Apple Retina) scaling factor;
        let num_pixels_in_ui = size_in_ui * ctx.pixels_per_point();
        let x_factor = num_pixels_in_ui.x / original_size.x;
        let y_factor = num_pixels_in_ui.y / original_size.y;
        let factor = x_factor.max(y_factor); // possible for these two to be slightly different..?
        if factor < 1.0 {
            let mut should_override_factor = false;
            if let Some(curr_factor) = context.current_edit_ref().resize_factor {
                if (curr_factor - factor).abs() > 0.01 {
                    // overwrite only when factor changes noticeably, to avoid constant-rescaling due to egui size jitters.
                    should_override_factor = true;
                }
            } else {
                should_override_factor = true;
            }
            if should_override_factor {
                context.override_resize_factor(factor);
                session.editor.execute_current_edit();
            }
        }

        let context = session.editor.current_edit_context_mut().unwrap();
        if let Some(ref result) = context.current_result {
            let mut mask = None;
            if let Some(term_index) = ui_state.selected_mask_term_index {
                mask = Some(
                    result.masked_edit_results[ui_state.selected_mask_index].mask_terms[term_index]
                        .clone(),
                );
            }

            let main_image_callback = MainImageCallback {
                image: result.final_image.clone(),
                mask,
                ui_max_rect: ui.max_rect(),
            };

            let main_image_rect = main_image_callback.image_ui_rect();
            let response = ui.allocate_rect(main_image_rect, egui::Sense::drag());
            ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                main_image_rect,
                main_image_callback,
            ));

            if ui_state.show_grid {
                draw_grid_impl(ui, main_image_rect, ui_state);
            }
            if let Some(term_index) = ui_state.selected_mask_term_index {
                let mut transient_edit = context.transient_edit_ref().clone();
                let primitive = &mut transient_edit.masked_edits[ui_state.selected_mask_index]
                    .mask
                    .terms[term_index]
                    .primitive;
                let should_commit = mask_primitive_control_points(
                    ui,
                    main_image_rect,
                    &response,
                    primitive,
                    &mut ui_state.mask_edit_state,
                );
                session.editor.update_transient_edit(transient_edit, true);
                if should_commit {
                    session.editor.commit_transient_edit(false);
                }
            }
        }
    });
}

fn image_crop_and_rotate(
    ctx: &egui::Context,
    ui: &mut Ui,
    session: &mut Session,
    ui_state: &mut AppUiState,
) {
    ui.centered_and_justified(|ui| {
        let context = session.editor.current_edit_context_ref().unwrap();
        let original_image = context.input_image();
        let mut transient_edit = context.transient_edit_ref().clone();

        let full_image_callback = ImageGeometryEditCallback {
            full_image: original_image.clone(),
            rotation_degrees: transient_edit.rotation_degrees.clone().unwrap_or(0.0),
            crop_rect: transient_edit
                .crop_rect
                .clone()
                .unwrap_or(Rectangle::regular()),
            ui_max_rect: ui.max_rect(),
        };

        let full_image_allocated_rect = full_image_callback.required_allocated_rect();
        let cropped_image_ui_rect = full_image_callback.cropped_image_ui_rect();
        let response = ui.allocate_rect(full_image_allocated_rect, egui::Sense::drag());

        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            full_image_allocated_rect,
            full_image_callback,
        ));

        let original_crop_rect = transient_edit
            .crop_rect
            .clone()
            .unwrap_or(Rectangle::regular());
        let original_rotation_degrees = transient_edit.rotation_degrees.clone().unwrap_or(0.0);
        let new_crop_rect = handle_crop_and_rotate_response(
            ctx,
            ui,
            &response,
            original_image.aspect_ratio(),
            cropped_image_ui_rect.clone(),
            original_crop_rect.clone(),
            original_rotation_degrees,
            ui_state,
        );
        draw_drag_handles(ui, cropped_image_ui_rect, ui_state);
        draw_grid_impl(ui, cropped_image_ui_rect, ui_state);

        if let Some(ref new_crop_rect) = new_crop_rect {
            handle_new_crop_rect(
                original_image.aspect_ratio(),
                &mut transient_edit,
                *new_crop_rect,
            );
            session.editor.update_transient_edit(transient_edit, false);
        }
    });
}

// returns whether pending changes to control points should be committed
fn mask_primitive_control_points(
    ui: &mut Ui,
    rect: egui::Rect,
    response: &egui::Response,
    primitive: &mut MaskPrimitive,
    mask_edit_state: &mut MaskEditState,
) -> bool {
    match primitive {
        MaskPrimitive::RadialGradient(ref mut m) => {
            radial_gradient_control_points(ui, rect, response, m, mask_edit_state)
        }
        MaskPrimitive::LinearGradient(ref mut m) => {
            linear_gradient_control_points(ui, rect, response, m, mask_edit_state)
        }
        _ => false,
    }
}

fn get_absolute_pos(rect: egui::Rect, (relative_x, relative_y): (f32, f32)) -> (f32, f32) {
    (
        rect.min.x + relative_x * rect.width(),
        rect.min.y + relative_y * rect.height(),
    )
}

fn get_relative_pos(rect: egui::Rect, (abs_x, abs_y): (f32, f32)) -> (f32, f32) {
    (
        (abs_x - rect.min.x) / rect.width(),
        (abs_y - rect.min.y) / rect.height(),
    )
}

fn draw_control_point_circle(ui: &mut Ui, rect: egui::Rect, center: (f32, f32)) {
    let width = rect.width().min(rect.height());
    let center = Pos2 {
        x: center.0,
        y: center.1,
    };
    ui.painter().circle_stroke(
        center,
        width * 0.01,
        Stroke {
            color: Color32::from_rgb(50, 50, 250),
            width: width * 0.004,
        },
    );
    ui.painter()
        .circle_filled(center, width * 0.008, Color32::from_gray(230));
}

// returns whether pending changes to control points should be committed
fn radial_gradient_control_points(
    ui: &mut Ui,
    rect: egui::Rect,
    response: &egui::Response,
    radial_gradient: &mut RadialGradientMask,
    mask_edit_state: &mut MaskEditState,
) -> bool {
    let center = vec2((radial_gradient.center_x, radial_gradient.center_y));
    let center_abs = vec2(get_absolute_pos(rect, center.xy()));
    let theta = radial_gradient.rotation;
    let rotate = |p: Vec2<f32>| {
        vec2((
            p.x * theta.cos() - p.y * theta.sin(),
            p.x * theta.sin() + p.y * theta.cos(),
        ))
    };
    let control_points = vec![
        center,
        center + vec2((radial_gradient.radius_x, 0.0)),
        center + vec2((-radial_gradient.radius_x, 0.0)),
        center + vec2((0.0, radial_gradient.radius_y)),
        center + vec2((0.0, -radial_gradient.radius_y)),
    ];
    for i in 0..control_points.len() {
        let p = control_points[i];
        let p_abs = vec2(get_absolute_pos(rect, p.xy()));
        let mut p_minus_center = p_abs - center_abs;
        p_minus_center = rotate(p_minus_center);
        let p_abs = p_minus_center + center_abs;
        if let Some(dragged_index) = mask_edit_state.dragged_control_point_index {
            if dragged_index == i {
                ui.output_mut(|out| out.cursor_icon = CursorIcon::Grabbing);
                if response.dragged() {
                    let mut new_abs_p = p_abs;
                    if let Some(hover_pos) = response.hover_pos() {
                        new_abs_p = pos2_to_vec2(hover_pos);
                    }
                    let new_p = vec2(get_relative_pos(rect, new_abs_p.xy()));
                    if i == 0 {
                        radial_gradient.center_x = new_p.x;
                        radial_gradient.center_y = new_p.y;
                    } else {
                        let new_p_minus_center_abs = new_abs_p - center_abs;
                        let rotation = new_p_minus_center_abs.y.atan2(new_p_minus_center_abs.x);
                        if i == 1 || i == 2 {
                            radial_gradient.radius_x =
                                new_p_minus_center_abs.length() / rect.width();
                        } else {
                            radial_gradient.radius_y =
                                new_p_minus_center_abs.length() / rect.height();
                        }
                        if i == 1 {
                            radial_gradient.rotation = rotation;
                        } else if i == 2 {
                            radial_gradient.rotation = rotation - PI;
                        } else if i == 3 {
                            radial_gradient.rotation = rotation - PI * 0.5;
                        } else if i == 4 {
                            radial_gradient.rotation = rotation - PI * 1.5;
                        }
                    }
                }
            }
        } else if let Some(hover_pos) = response.hover_pos() {
            let diff = p_abs - vec2((hover_pos.x, hover_pos.y));
            let dist = diff.length();
            if dist < rect.width().min(rect.height()) * 0.012 {
                ui.output_mut(|out| out.cursor_icon = CursorIcon::Grab);
                if response.drag_started() {
                    mask_edit_state.dragged_control_point_index = Some(i);
                }
            }
        }
        draw_control_point_circle(ui, rect, p_abs.xy());
    }
    if response.drag_stopped() {
        mask_edit_state.dragged_control_point_index = None;
        return true;
    }
    false
}

// returns whether pending changes to control points should be committed
fn linear_gradient_control_points(
    ui: &mut Ui,
    rect: egui::Rect,
    response: &egui::Response,
    linear_gradient: &mut LinearGradientMask,
    mask_edit_state: &mut MaskEditState,
) -> bool {
    let begin = vec2((linear_gradient.begin_x, linear_gradient.begin_y));
    let saturated = vec2((linear_gradient.saturate_x, linear_gradient.saturate_y));
    let begin_abs = vec2(get_absolute_pos(rect, begin.xy()));
    let saturated_abs = vec2(get_absolute_pos(rect, saturated.xy()));
    let middle_abs = (begin_abs + saturated_abs) / 2.0;

    let control_points = vec![begin_abs, saturated_abs, middle_abs];

    for i in 0..control_points.len() {
        let p_abs = control_points[i];
        if let Some(dragged_index) = mask_edit_state.dragged_control_point_index {
            if dragged_index == i {
                ui.output_mut(|out| out.cursor_icon = CursorIcon::Grabbing);
                if response.dragged() {
                    let mut new_abs_p = p_abs;
                    if let Some(hover_pos) = response.hover_pos() {
                        new_abs_p = pos2_to_vec2(hover_pos);
                    }
                    let new_p = vec2(get_relative_pos(rect, new_abs_p.xy()));
                    if i == 0 {
                        linear_gradient.begin_x = new_p.x;
                        linear_gradient.begin_y = new_p.y;
                    } else if i == 1 {
                        linear_gradient.saturate_x = new_p.x;
                        linear_gradient.saturate_y = new_p.y;
                    } else if i == 2 {
                        let mut delta = new_abs_p - p_abs;
                        delta = delta / vec2((rect.width(), rect.height()));
                        linear_gradient.begin_x += delta.x;
                        linear_gradient.begin_y += delta.y;
                        linear_gradient.saturate_x += delta.x;
                        linear_gradient.saturate_y += delta.y;
                    }
                }
            }
        } else if let Some(hover_pos) = response.hover_pos() {
            let diff = p_abs - vec2((hover_pos.x, hover_pos.y));
            let dist = diff.length();
            if dist < rect.width().min(rect.height()) * 0.012 {
                ui.output_mut(|out| out.cursor_icon = CursorIcon::Grab);
                if response.drag_started() {
                    mask_edit_state.dragged_control_point_index = Some(i);
                }
            }
        }
        draw_control_point_circle(ui, rect, p_abs.xy());
    }

    let painter = ui.painter_at(rect);
    let normal = (begin_abs - saturated_abs).normalized();
    let line = vec2((-normal.y, normal.x));
    let len = rect.width() + rect.height();
    let stroke = Stroke {
        width: len * 0.001,
        color: Color32::WHITE,
    };
    for p in control_points.iter() {
        let start = *p + line * len;
        let end = *p + line * -len;
        painter.line_segment(
            [
                Pos2 {
                    x: start.x,
                    y: start.y,
                },
                Pos2 { x: end.x, y: end.y },
            ],
            stroke,
        );
    }

    if response.drag_stopped() {
        mask_edit_state.dragged_control_point_index = None;
        return true;
    }
    false
}

fn find_edge_or_corner(pos: egui::Pos2, rect: egui::Rect) -> Option<CropDragEdgeOrCorner> {
    let mut x_selected: Option<f32> = None;
    let mut y_selected: Option<f32> = None;
    let threshold = rect.width().min(rect.height()) * 0.05;
    for t in [0.0 as f32, 1.0] {
        let x_dist = (rect.min.x + rect.width() * t - pos.x).abs();
        let y_dist = (rect.min.y + rect.height() * t - pos.y).abs();
        if x_dist < threshold {
            x_selected = Some(t);
        }
        if y_dist < threshold {
            y_selected = Some(t);
        }
    }
    if let (Some(x), Some(y)) = (x_selected, y_selected) {
        if y == 0.0 && x == 0.0 {
            return Some(CropDragEdgeOrCorner::TopLeft);
        } else if y == 0.0 && x == 1.0 {
            return Some(CropDragEdgeOrCorner::TopRight);
        } else if y == 1.0 && x == 0.0 {
            return Some(CropDragEdgeOrCorner::BottomLeft);
        } else if y == 1.0 && x == 1.0 {
            return Some(CropDragEdgeOrCorner::BottomRight);
        }
    }
    if let Some(x) = x_selected {
        if x == 0.0 {
            return Some(CropDragEdgeOrCorner::Left);
        } else {
            return Some(CropDragEdgeOrCorner::Right);
        }
    }
    if let Some(y) = y_selected {
        if y == 0.0 {
            return Some(CropDragEdgeOrCorner::Top);
        } else {
            return Some(CropDragEdgeOrCorner::Bottom);
        }
    }
    None
}

fn set_edge_or_corner_cursor(ui: &mut Ui, edge_or_corner: CropDragEdgeOrCorner) {
    match edge_or_corner {
        CropDragEdgeOrCorner::Left | CropDragEdgeOrCorner::Right => {
            ui.output_mut(|out| out.cursor_icon = CursorIcon::ResizeHorizontal);
        }
        CropDragEdgeOrCorner::Top | CropDragEdgeOrCorner::Bottom => {
            ui.output_mut(|out| out.cursor_icon = CursorIcon::ResizeVertical);
        }
        CropDragEdgeOrCorner::TopLeft | CropDragEdgeOrCorner::BottomRight => {
            ui.output_mut(|out| out.cursor_icon = CursorIcon::ResizeNwSe);
        }
        CropDragEdgeOrCorner::TopRight | CropDragEdgeOrCorner::BottomLeft => {
            ui.output_mut(|out| out.cursor_icon = CursorIcon::ResizeNeSw);
        }
    }
}

fn handle_crop_and_rotate_response(
    ctx: &egui::Context,
    ui: &mut Ui,
    response: &egui::Response,
    original_image_aspect_ratio: f32,
    original_ui_crop_rect: egui::Rect,
    original_crop_rect: Rectangle,
    original_rotation_degrees: f32,
    ui_state: &mut AppUiState,
) -> Option<Rectangle> {
    if let Some(ref edge_or_corner) = ui_state.crop_drag_state.edge_or_corner {
        set_edge_or_corner_cursor(ui, *edge_or_corner);
        if response.dragged() {
            let mut new_crop_rect = original_crop_rect;
            let delta = response.drag_delta();
            let mut delta = vec2((
                delta.x / (original_ui_crop_rect.width() / original_crop_rect.size.x),
                delta.y / (original_ui_crop_rect.height() / original_crop_rect.size.y),
            ));
            // * 2.0 to counter the fact that this causes the crop rect center to move, which causes the full image to move
            delta = delta * 2.0;

            let rotation_mat = get_rotation_mat(-original_rotation_degrees.to_radians());

            if edge_or_corner.has_left() && delta.x < 0.0 {
                let delta_bounds = get_crop_rect_upscale_bounds(
                    original_rotation_degrees,
                    new_crop_rect,
                    vec2((-1.0, 0.0)),
                    original_image_aspect_ratio,
                );
                let mut neg_x = -delta.x;
                neg_x = neg_x.max(delta_bounds.0).min(delta_bounds.1);
                delta.x = -neg_x;
            }
            if edge_or_corner.has_right() && delta.x > 0.0 {
                let delta_bounds = get_crop_rect_upscale_bounds(
                    original_rotation_degrees,
                    new_crop_rect,
                    vec2((1.0, 0.0)),
                    original_image_aspect_ratio,
                );
                delta.x = delta.x.max(delta_bounds.0).min(delta_bounds.1);
            }
            if edge_or_corner.has_top() && delta.y < 0.0 {
                let delta_bounds = get_crop_rect_upscale_bounds(
                    original_rotation_degrees,
                    new_crop_rect,
                    vec2((0.0, -1.0)),
                    original_image_aspect_ratio,
                );
                let mut neg_y = -delta.y;
                neg_y = neg_y.max(delta_bounds.0).min(delta_bounds.1);
                delta.y = -neg_y;
            }
            if edge_or_corner.has_bottom() {
                let delta_bounds = get_crop_rect_upscale_bounds(
                    original_rotation_degrees,
                    new_crop_rect,
                    vec2((0.0, 1.0)),
                    original_image_aspect_ratio,
                );
                delta.y = delta.y.max(delta_bounds.0).min(delta_bounds.1);
            }

            if edge_or_corner.is_corner() {
                let mut should_keep = false;
                match edge_or_corner {
                    CropDragEdgeOrCorner::TopLeft | CropDragEdgeOrCorner::BottomRight => {
                        should_keep = delta.x.signum() == delta.y.signum();
                    }
                    CropDragEdgeOrCorner::TopRight | CropDragEdgeOrCorner::BottomLeft => {
                        should_keep = delta.x.signum() == -delta.y.signum();
                    }
                    _ => {}
                }
                if should_keep {
                    let cropped_aspect_ratio = new_crop_rect.size.y / new_crop_rect.size.x;
                    let min_abs_x = delta.x.abs().min(delta.y.abs() / cropped_aspect_ratio);
                    delta.x = min_abs_x * delta.x.signum();
                    delta.y = min_abs_x * delta.y.signum() * cropped_aspect_ratio;
                } else {
                    delta = vec2((0.0, 0.0));
                }
            }

            let min_crop_size = 0.001;

            if edge_or_corner.has_left() {
                delta.x = delta.x.min(new_crop_rect.size.x - min_crop_size);
            }
            if edge_or_corner.has_right() {
                delta.x = delta.x.max(min_crop_size - new_crop_rect.size.x);
            }
            if edge_or_corner.has_top() {
                delta.y = delta.y.min(new_crop_rect.size.y - min_crop_size);
            }
            if edge_or_corner.has_bottom() {
                delta.y = delta.y.max(min_crop_size - new_crop_rect.size.y);
            }

            if edge_or_corner.has_left() {
                new_crop_rect.size.x -= delta.x;
            }
            if edge_or_corner.has_right() {
                new_crop_rect.size.x += delta.x;
            }
            if edge_or_corner.has_top() {
                new_crop_rect.size.y -= delta.y;
            }
            if edge_or_corner.has_bottom() {
                new_crop_rect.size.y += delta.y;
            }

            if edge_or_corner.has_left() || edge_or_corner.has_right() {
                let mut dir = vec2((1.0, 0.0));
                dir.x /= original_image_aspect_ratio;
                dir = rotation_mat * dir;
                dir.x *= original_image_aspect_ratio;
                new_crop_rect.center = new_crop_rect.center + dir * 0.5 * delta.x;
            }
            if edge_or_corner.has_top() || edge_or_corner.has_bottom() {
                let mut dir = vec2((0.0, 1.0));
                dir.x /= original_image_aspect_ratio;
                dir = rotation_mat * dir;
                dir.x *= original_image_aspect_ratio;
                new_crop_rect.center = new_crop_rect.center + dir * 0.5 * delta.y;
            }

            return Some(new_crop_rect);
        } else if response.drag_stopped() {
            ui_state.crop_drag_state.edge_or_corner = None;
        }
    } else if ui_state.crop_drag_state.translation {
        ui.output_mut(|out| out.cursor_icon = CursorIcon::Grabbing);
        if response.dragged() {
            let delta = response.drag_delta();
            let mut delta = vec2((delta.x, delta.y));

            // the image is dragged, relative to a fixed crop rect
            delta = delta * -1.0;

            delta = delta
                / vec2((
                    original_ui_crop_rect.width() / original_crop_rect.size.x,
                    original_ui_crop_rect.height() / original_crop_rect.size.y,
                ));

            delta.x /= original_image_aspect_ratio;

            if delta.x != 0.0 || delta.y != 0.0 {
                let delta_bounds = get_crop_rect_translation_bounds(
                    original_rotation_degrees,
                    original_crop_rect,
                    original_image_aspect_ratio,
                );

                if delta.x < 0.0 {
                    let mut neg_x = -delta.x;
                    neg_x = neg_x.max(delta_bounds[0].0).min(delta_bounds[0].1);
                    delta.x = -neg_x;
                } else if delta.x > 0.0 {
                    delta.x = delta.x.max(delta_bounds[1].0).min(delta_bounds[1].1)
                }

                if delta.y < 0.0 {
                    let mut neg_y = -delta.y;
                    neg_y = neg_y.max(delta_bounds[2].0).min(delta_bounds[2].1);
                    delta.y = -neg_y;
                } else if delta.y > 0.0 {
                    delta.y = delta.y.max(delta_bounds[3].0).min(delta_bounds[3].1)
                }
            }

            let rotation_mat = get_rotation_mat(-original_rotation_degrees.to_radians());
            delta = rotation_mat * delta;

            delta.x *= original_image_aspect_ratio;

            let mut new_crop_rect = original_crop_rect;
            new_crop_rect.center = new_crop_rect.center + delta;
            return Some(new_crop_rect);
        } else if response.drag_stopped() {
            ui_state.crop_drag_state.translation = false;
        }
    } else {
        if let Some(hover_pos) = response.hover_pos() {
            if let Some(edge_or_corner) = find_edge_or_corner(hover_pos, original_ui_crop_rect) {
                set_edge_or_corner_cursor(ui, edge_or_corner);
                if response.drag_started() {
                    ui_state.crop_drag_state.edge_or_corner = Some(edge_or_corner);
                }
            } else {
                if original_ui_crop_rect.contains(hover_pos) {
                    ui.output_mut(|out| out.cursor_icon = CursorIcon::Grab);
                    if response.drag_started() {
                        ui_state.crop_drag_state.translation = true;
                    }
                }
            }
        }
    }
    None
}

fn draw_drag_handles(ui: &mut Ui, ui_crop_rect: egui::Rect, ui_state: &mut AppUiState) {
    let thickness = ui_crop_rect.width().min(ui_crop_rect.height()) * 0.005;
    let thickness = thickness.max(3.0);
    let length: egui::Vec2 = ui_crop_rect.size() * 0.1;
    let stroke_non_selected = egui::Stroke::new(thickness, Color32::from_gray(250));
    let stroke_selected = egui::Stroke::new(thickness, Color32::from_rgb(50, 150, 200));

    let selected_edge_or_corner = ui_state.crop_drag_state.edge_or_corner.clone();

    let top_y = ui_crop_rect.min.y - thickness * 0.5;
    let bottom_y = ui_crop_rect.max.y + thickness * 0.5;
    let left_x = ui_crop_rect.min.x - thickness * 0.5;
    let right_x = ui_crop_rect.max.x + thickness * 0.5;

    let stroke = if selected_edge_or_corner == Some(CropDragEdgeOrCorner::Top) {
        stroke_selected
    } else {
        stroke_non_selected
    };
    ui.painter().hline(
        egui::Rangef::new(
            ui_crop_rect.center().x - length.x * 0.5,
            ui_crop_rect.center().x + length.x * 0.5,
        ),
        top_y,
        stroke,
    );

    let stroke = if selected_edge_or_corner == Some(CropDragEdgeOrCorner::Bottom) {
        stroke_selected
    } else {
        stroke_non_selected
    };
    ui.painter().hline(
        egui::Rangef::new(
            ui_crop_rect.center().x - length.x * 0.5,
            ui_crop_rect.center().x + length.x * 0.5,
        ),
        bottom_y,
        stroke,
    );

    let stroke = if selected_edge_or_corner == Some(CropDragEdgeOrCorner::Left) {
        stroke_selected
    } else {
        stroke_non_selected
    };
    ui.painter().vline(
        left_x,
        egui::Rangef::new(
            ui_crop_rect.center().y - length.y * 0.5,
            ui_crop_rect.center().y + length.y * 0.5,
        ),
        stroke,
    );

    let stroke = if selected_edge_or_corner == Some(CropDragEdgeOrCorner::Right) {
        stroke_selected
    } else {
        stroke_non_selected
    };
    ui.painter().vline(
        right_x,
        egui::Rangef::new(
            ui_crop_rect.center().y - length.y * 0.5,
            ui_crop_rect.center().y + length.y * 0.5,
        ),
        stroke,
    );

    let stroke = if selected_edge_or_corner == Some(CropDragEdgeOrCorner::TopLeft) {
        stroke_selected
    } else {
        stroke_non_selected
    };
    ui.painter().hline(
        egui::Rangef::new(
            ui_crop_rect.min.x - thickness,
            ui_crop_rect.min.x - thickness + length.x * 0.5,
        ),
        top_y,
        stroke,
    );
    ui.painter().vline(
        left_x,
        egui::Rangef::new(
            ui_crop_rect.min.y - thickness,
            ui_crop_rect.min.y - thickness + length.y * 0.5,
        ),
        stroke,
    );

    let stroke = if selected_edge_or_corner == Some(CropDragEdgeOrCorner::TopRight) {
        stroke_selected
    } else {
        stroke_non_selected
    };
    ui.painter().hline(
        egui::Rangef::new(
            ui_crop_rect.max.x + thickness - length.x * 0.5,
            ui_crop_rect.max.x + thickness,
        ),
        top_y,
        stroke,
    );
    ui.painter().vline(
        right_x,
        egui::Rangef::new(
            ui_crop_rect.min.y - thickness,
            ui_crop_rect.min.y - thickness + length.y * 0.5,
        ),
        stroke,
    );

    let stroke = if selected_edge_or_corner == Some(CropDragEdgeOrCorner::BottomLeft) {
        stroke_selected
    } else {
        stroke_non_selected
    };
    ui.painter().hline(
        egui::Rangef::new(
            ui_crop_rect.min.x - thickness,
            ui_crop_rect.min.x - thickness + length.x * 0.5,
        ),
        bottom_y,
        stroke,
    );
    ui.painter().vline(
        left_x,
        egui::Rangef::new(
            ui_crop_rect.max.y + thickness - length.y * 0.5,
            ui_crop_rect.max.y + thickness,
        ),
        stroke,
    );

    let stroke = if selected_edge_or_corner == Some(CropDragEdgeOrCorner::BottomRight) {
        stroke_selected
    } else {
        stroke_non_selected
    };
    ui.painter().hline(
        egui::Rangef::new(
            ui_crop_rect.max.x + thickness - length.x * 0.5,
            ui_crop_rect.max.x + thickness,
        ),
        bottom_y,
        stroke,
    );
    ui.painter().vline(
        right_x,
        egui::Rangef::new(
            ui_crop_rect.max.y + thickness - length.y * 0.5,
            ui_crop_rect.max.y + thickness,
        ),
        stroke,
    );
}

fn draw_grid_impl(ui: &mut Ui, ui_crop_rect: egui::Rect, ui_state: &mut AppUiState) {
    let width = ui_crop_rect.width().min(ui_crop_rect.height()) * 0.001;
    let width = width.max(1.0);
    let stroke_non_selected = egui::Stroke::new(width, Color32::from_gray(200));
    let stroke_selected = egui::Stroke::new(width * 2.0, Color32::from_rgb(50, 150, 200));
    for t in [0.0, 1.0, 2.0, 3.0] {
        let mut stroke = stroke_non_selected;
        if let Some(ref edge_or_corner) = ui_state.crop_drag_state.edge_or_corner {
            if (t == 0.0 && edge_or_corner.has_top()) || (t == 3.0 && edge_or_corner.has_bottom()) {
                stroke = stroke_selected;
            }
        }
        ui.painter().hline(
            egui::Rangef::new(ui_crop_rect.min.x, ui_crop_rect.max.x),
            ui_crop_rect.min.y + ui_crop_rect.height() * t / 3.0,
            stroke,
        );

        let mut stroke = stroke_non_selected;
        if let Some(ref edge_or_corner) = ui_state.crop_drag_state.edge_or_corner {
            if (t == 0.0 && edge_or_corner.has_left()) || (t == 3.0 && edge_or_corner.has_right()) {
                stroke = stroke_selected;
            }
        }
        ui.painter().vline(
            ui_crop_rect.min.x + ui_crop_rect.width() * t / 3.0,
            egui::Rangef::new(ui_crop_rect.min.y, ui_crop_rect.max.y),
            stroke,
        );
    }
}

pub fn get_ui_crop_rect(full_image_rect: egui::Rect, crop_rect: Rectangle) -> egui::Rect {
    egui::Rect::from_center_size(
        full_image_rect.center(),
        full_image_rect.size() * egui::vec2(crop_rect.size.x, crop_rect.size.y),
    )
}

pub fn get_image_size_in_ui(ui: &Ui, image_aspect_ratio: f32) -> egui::Vec2 {
    get_max_image_size(
        image_aspect_ratio,
        ui.available_width(),
        ui.available_height(),
    )
}
