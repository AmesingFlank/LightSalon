use std::mem::size_of;
use std::sync::Arc;
use std::{collections::HashMap, num::NonZeroU64};

use eframe::egui::Ui;
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

use super::MainImageCallback;

pub fn main_image(ui: &mut Ui, session: &mut Session) {
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
                ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                    rect,
                    MainImageCallback {
                        image: image.clone(),
                    },
                ));
            });
        }
    }
}
