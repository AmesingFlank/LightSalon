use std::mem::size_of;
use std::sync::Arc;
use std::{collections::HashMap, num::NonZeroU64};

use eframe::egui::Ui;
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

use super::widgets::MainImageCallback;
use super::{AppUiState, EditorPanel};

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
                let (rect, response) = ui.allocate_exact_size(size, egui::Sense::drag());
                //let painter = egui::Painter::new(ctx.clone(), ui.layer_id(), rect);
                let draw_grid = ui_state.show_grid;
                ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                    rect,
                    MainImageCallback {
                        image: image.clone(),
                    },
                ));
                if ui_state.editor_panel == EditorPanel::CropAndRotate {
                    draw_grid_impl(ui, rect);
                    draw_drag_handles(ui, rect);
                }
                if draw_grid {
                    draw_grid_impl(ui, rect);
                }
            });
        }
    }
}

fn draw_drag_handles(ui: &mut Ui, rect: egui::Rect) {
    let width = rect.width().min(rect.height()) * 0.005;
    let length = rect.width().min(rect.height()) * 0.1;
    let stroke = egui::Stroke::new(width, Color32::from_gray(250));

    for y in [rect.min.y - width * 0.5, rect.max.y + width * 0.5] {
        ui.painter().hline(
            egui::Rangef::new(rect.min.x - width, rect.min.x - width + length * 0.5),
            y,
            stroke,
        );
        ui.painter().hline(
            egui::Rangef::new(
                rect.min.x + rect.width() * 0.5 - length * 0.5,
                rect.min.x + rect.width() * 0.5 + length * 0.5,
            ),
            y,
            stroke,
        );
        ui.painter().hline(
            egui::Rangef::new(rect.max.x + width - length * 0.5, rect.max.x + width),
            y,
            stroke,
        );
    }

    for x in [rect.min.x - width * 0.5, rect.max.x + width * 0.5] {
        ui.painter().vline(
            x,
            egui::Rangef::new(rect.min.y - width, rect.min.y - width + length * 0.5),
            stroke,
        );
        ui.painter().vline(
            x,
            egui::Rangef::new(
                rect.min.y + rect.height() * 0.5 - length * 0.5,
                rect.min.y + rect.height() * 0.5 + length * 0.5,
            ),
            stroke,
        );
        ui.painter().vline(
            x,
            egui::Rangef::new(rect.max.y + width - length * 0.5, rect.max.y + width),
            stroke,
        );
    }
}

fn draw_grid_impl(ui: &mut Ui, rect: egui::Rect) {
    let width = rect.width().min(rect.height()) * 0.001;
    let stroke = egui::Stroke::new(width, Color32::from_gray(200));
    for t in [0.0, 1.0, 2.0, 3.0] {
        ui.painter().hline(
            egui::Rangef::new(rect.min.x, rect.max.x),
            rect.min.y + rect.height() * t / 3.0,
            stroke,
        );
        ui.painter().vline(
            rect.min.x + rect.width() * t / 3.0,
            egui::Rangef::new(rect.min.y, rect.max.y),
            stroke,
        );
    }
}
