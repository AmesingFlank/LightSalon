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
use super::AppUiState;

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
                if draw_grid {
                    let stroke = egui::Stroke::new(rect.width() * 0.002, Color32::from_gray(200));
                    ui.painter().hline(
                        egui::Rangef::new(rect.min.x, rect.max.x),
                        rect.min.y + rect.height() / 3.0,
                        stroke,
                    );
                    ui.painter().hline(
                        egui::Rangef::new(rect.min.x, rect.max.x),
                        rect.min.y + rect.height() * 2.0 / 3.0,
                        stroke,
                    );
                    ui.painter().vline(
                        rect.min.x + rect.width() / 3.0,
                        egui::Rangef::new(rect.min.y, rect.max.y),
                        stroke,
                    );
                    ui.painter().vline(
                        rect.min.x + rect.width() * 2.0 / 3.0,
                        egui::Rangef::new(rect.min.y, rect.max.y),
                        stroke,
                    );
                }
            });
        }
    }
}
