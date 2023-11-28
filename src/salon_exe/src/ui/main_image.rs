use std::mem::size_of;
use std::sync::Arc;
use std::{collections::HashMap, num::NonZeroU64};

use eframe::egui::{CursorIcon, Ui};
use eframe::epaint::{Color32, Pos2, Stroke};
use eframe::{egui, egui_wgpu};
use salon_core::buffer::{Buffer, BufferProperties, RingBuffer};
use salon_core::image::Image;
use salon_core::runtime::{
    BindGroupDescriptor, BindGroupDescriptorKey, BindGroupEntry, BindGroupManager, BindingResource,
    Runtime,
};
use salon_core::sampler::Sampler;
use salon_core::session::Session;
use salon_core::shader::{Shader, ShaderLibraryModule};
use serde_json::de;

use super::widgets::MainImageCallback;
use super::{AppUiState, CropDragEdgeOrCorner, EditorPanel};

pub fn main_image(
    ctx: &egui::Context,
    ui: &mut Ui,
    session: &mut Session,
    ui_state: &mut AppUiState,
) {
    if let Some(ref result) = session.editor.current_result {
        let max_x = ui.available_width();
        let max_y = ui.available_height();
        let ui_aspect_ratio = max_y / max_x;

        if let Some(ref image) = result.final_image.clone() {
            let image_aspect_ratio = image.aspect_ratio();

            let size = if image_aspect_ratio >= ui_aspect_ratio {
                egui::Vec2 {
                    x: max_y / image_aspect_ratio,
                    y: max_y,
                }
            } else {
                egui::Vec2 {
                    x: max_x,
                    y: max_x * image_aspect_ratio,
                }
            };

            ui.centered_and_justified(|ui| {
                let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
                //let painter = egui::Painter::new(ctx.clone(), ui.layer_id(), rect);
                let draw_grid = ui_state.show_grid;
                ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                    rect,
                    MainImageCallback {
                        image: image.clone(),
                    },
                ));
                if ui_state.editor_panel == EditorPanel::CropAndRotate {
                    let original_rect = rect;
                    let mut curr_rect = original_rect;
                    if let Some(ref rect) = ui_state.crop_drag_state.rect {
                        curr_rect = rect.clone();
                    }
                    handle_crop_and_rotate_response(
                        ui,
                        &response,
                        curr_rect,
                        original_rect,
                        ui_state,
                    );
                    draw_drag_handles(ui, curr_rect, ui_state);
                    draw_grid_impl(ui, curr_rect, ui_state);
                }
                if draw_grid {
                    draw_grid_impl(ui, rect, ui_state);
                }
            });
        }
    }
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
    ui: &mut Ui,
    response: &egui::Response,
    curr_rect: egui::Rect,
    original_rect: egui::Rect,
    ui_state: &mut AppUiState,
) {
    if let Some(ref edge_or_corner) = ui_state.crop_drag_state.edge_or_corner {
        set_edge_or_corner_cursor(ui, *edge_or_corner);
        if response.dragged() {
            let aspect_ratio = curr_rect.height() / curr_rect.width();
            let new_rect = ui_state.crop_drag_state.rect.as_mut().unwrap();
            let delta = response.drag_delta();
            if let Some(ref edge_or_corner) = ui_state.crop_drag_state.edge_or_corner {
                match edge_or_corner {
                    CropDragEdgeOrCorner::Left => {
                        new_rect.min.x += delta.x;
                    }
                    CropDragEdgeOrCorner::Right => {
                        new_rect.max.x += delta.x;
                    }
                    CropDragEdgeOrCorner::Top => {
                        new_rect.min.y += delta.y;
                    }
                    CropDragEdgeOrCorner::Bottom => {
                        new_rect.max.y += delta.y;
                    }
                    CropDragEdgeOrCorner::TopLeft => {
                        if delta.x.signum() == delta.y.signum() {
                            let x_abs = delta.x.abs().min(delta.y.abs() / aspect_ratio);
                            new_rect.min.x += x_abs * delta.x.signum();
                            new_rect.min.y += x_abs * aspect_ratio * delta.y.signum();
                        }
                    }
                    CropDragEdgeOrCorner::BottomRight => {
                        if delta.x.signum() == delta.y.signum() {
                            let x_abs = delta.x.abs().min(delta.y.abs() / aspect_ratio);
                            new_rect.max.x += x_abs * delta.x.signum();
                            new_rect.max.y += x_abs * aspect_ratio * delta.y.signum();
                        }
                    }
                    CropDragEdgeOrCorner::TopRight => {
                        if delta.x.signum() == -delta.y.signum() {
                            let x_abs = delta.x.abs().min(delta.y.abs() / aspect_ratio);
                            new_rect.max.x += x_abs * delta.x.signum();
                            new_rect.min.y += x_abs * aspect_ratio * delta.y.signum();
                        }
                    }
                    CropDragEdgeOrCorner::BottomLeft => {
                        if delta.x.signum() == -delta.y.signum() {
                            let x_abs = delta.x.abs().min(delta.y.abs() / aspect_ratio);
                            new_rect.min.x += x_abs * delta.x.signum();
                            new_rect.max.y += x_abs * aspect_ratio * delta.y.signum();
                        }
                    }
                    _ => {}
                }
            }

            new_rect.min.x = new_rect.min.x.min(curr_rect.max.x);
            new_rect.min.y = new_rect.min.y.min(curr_rect.max.y);
            new_rect.max.x = new_rect.max.x.max(curr_rect.min.x);
            new_rect.max.y = new_rect.max.y.max(curr_rect.min.y);

            new_rect.min.x = new_rect.min.x.max(original_rect.min.x);
            new_rect.min.y = new_rect.min.y.max(original_rect.min.y);
            new_rect.max.x = new_rect.max.x.min(original_rect.max.x);
            new_rect.max.y = new_rect.max.y.min(original_rect.max.y);
        } else if response.drag_released() {
            ui_state.crop_drag_state.edge_or_corner = None;
        }
    } else if ui_state.crop_drag_state.translation {
        ui.output_mut(|out| out.cursor_icon = CursorIcon::Grabbing);
        if response.dragged() {
            let mut delta = response.drag_delta();
            delta.x = delta.x.min(original_rect.max.x - curr_rect.max.x);
            delta.x = delta.x.max(original_rect.min.x - curr_rect.min.x);
            delta.y = delta.y.min(original_rect.max.y - curr_rect.max.y);
            delta.y = delta.y.max(original_rect.min.y - curr_rect.min.y);

            let new_rect = ui_state.crop_drag_state.rect.as_mut().unwrap();
            new_rect.min.x += delta.x;
            new_rect.max.x += delta.x;
            new_rect.min.y += delta.y;
            new_rect.max.y += delta.y;
        } else if response.drag_released() {
            ui_state.crop_drag_state.translation = false;
        }
    } else {
        if let Some(hover_pos) = response.hover_pos() {
            if let Some(edge_or_corner) = find_edge_or_corner(hover_pos, curr_rect) {
                set_edge_or_corner_cursor(ui, edge_or_corner);
                if response.drag_started() {
                    ui_state.crop_drag_state.edge_or_corner = Some(edge_or_corner);
                    ui_state.crop_drag_state.rect = Some(curr_rect);
                }
            } else {
                if curr_rect.contains(hover_pos) {
                    ui.output_mut(|out| out.cursor_icon = CursorIcon::Grab);
                    if response.drag_started() {
                        ui_state.crop_drag_state.translation = true;
                        ui_state.crop_drag_state.rect = Some(curr_rect);
                    }
                }
            }
        }
    }
}

fn draw_drag_handles(ui: &mut Ui, rect: egui::Rect, ui_state: &mut AppUiState) {
    let width = rect.width().min(rect.height()) * 0.005;
    let length = rect.width().min(rect.height()) * 0.1;
    let stroke_non_selected = egui::Stroke::new(width, Color32::from_gray(250));
    let stroke_selected = egui::Stroke::new(width, Color32::from_rgb(50, 150, 200));

    let selected_edge_or_corner = ui_state.crop_drag_state.edge_or_corner.clone();

    let top_y = rect.min.y - width * 0.5;
    let bottom_y = rect.max.y + width * 0.5;
    let left_x = rect.min.x - width * 0.5;
    let right_x = rect.max.x + width * 0.5;

    let stroke = if selected_edge_or_corner == Some(CropDragEdgeOrCorner::Top) {
        stroke_selected
    } else {
        stroke_non_selected
    };
    ui.painter().hline(
        egui::Rangef::new(
            rect.min.x + rect.width() * 0.5 - length * 0.5,
            rect.min.x + rect.width() * 0.5 + length * 0.5,
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
            rect.min.x + rect.width() * 0.5 - length * 0.5,
            rect.min.x + rect.width() * 0.5 + length * 0.5,
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
            rect.min.y + rect.height() * 0.5 - length * 0.5,
            rect.min.y + rect.height() * 0.5 + length * 0.5,
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
            rect.min.y + rect.height() * 0.5 - length * 0.5,
            rect.min.y + rect.height() * 0.5 + length * 0.5,
        ),
        stroke,
    );

    let stroke = if selected_edge_or_corner == Some(CropDragEdgeOrCorner::TopLeft) {
        stroke_selected
    } else {
        stroke_non_selected
    };
    ui.painter().hline(
        egui::Rangef::new(rect.min.x - width, rect.min.x - width + length * 0.5),
        top_y,
        stroke,
    );
    ui.painter().vline(
        left_x,
        egui::Rangef::new(rect.min.y - width, rect.min.y - width + length * 0.5),
        stroke,
    );

    let stroke = if selected_edge_or_corner == Some(CropDragEdgeOrCorner::TopRight) {
        stroke_selected
    } else {
        stroke_non_selected
    };
    ui.painter().hline(
        egui::Rangef::new(rect.max.x + width - length * 0.5, rect.max.x + width),
        top_y,
        stroke,
    );
    ui.painter().vline(
        right_x,
        egui::Rangef::new(rect.min.y - width, rect.min.y - width + length * 0.5),
        stroke,
    );

    let stroke = if selected_edge_or_corner == Some(CropDragEdgeOrCorner::BottomLeft) {
        stroke_selected
    } else {
        stroke_non_selected
    };
    ui.painter().hline(
        egui::Rangef::new(rect.min.x - width, rect.min.x - width + length * 0.5),
        bottom_y,
        stroke,
    );
    ui.painter().vline(
        left_x,
        egui::Rangef::new(rect.max.y + width - length * 0.5, rect.max.y + width),
        stroke,
    );

    let stroke = if selected_edge_or_corner == Some(CropDragEdgeOrCorner::BottomRight) {
        stroke_selected
    } else {
        stroke_non_selected
    };
    ui.painter().hline(
        egui::Rangef::new(rect.max.x + width - length * 0.5, rect.max.x + width),
        bottom_y,
        stroke,
    );
    ui.painter().vline(
        right_x,
        egui::Rangef::new(rect.max.y + width - length * 0.5, rect.max.y + width),
        stroke,
    );
}

fn draw_grid_impl(ui: &mut Ui, rect: egui::Rect, ui_state: &mut AppUiState) {
    let width = rect.width().min(rect.height()) * 0.001;
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
            egui::Rangef::new(rect.min.x, rect.max.x),
            rect.min.y + rect.height() * t / 3.0,
            stroke,
        );

        let mut stroke = stroke_non_selected;
        if let Some(ref edge_or_corner) = ui_state.crop_drag_state.edge_or_corner {
            if (t == 0.0 && edge_or_corner.has_left()) || (t == 3.0 && edge_or_corner.has_right()) {
                stroke = stroke_selected;
            }
        }
        ui.painter().vline(
            rect.min.x + rect.width() * t / 3.0,
            egui::Rangef::new(rect.min.y, rect.max.y),
            stroke,
        );
    }
}
