use crate::ui;
use eframe::egui;
use pepe_core::{engine::Engine, runtime::Runtime};
use std::{num::NonZeroU64, sync::Arc};

use eframe::{
    egui_wgpu::wgpu::util::DeviceExt,
    egui_wgpu::{self, wgpu},
};

pub struct App {
    engine: Engine,
    img: Arc<pepe_core::image::Image>,
}

impl App {
    pub fn main() {
        env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(1920.0, 1080.0)),
            renderer: eframe::Renderer::Wgpu,
            ..Default::default()
        };
        let _ = eframe::run_native("PEPE", options, Box::new(|_cc| Box::new(App::new(_cc))));
    }
    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(300.0), egui::Sense::drag());

        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            ui::main_image::MainImageCallback {
                arg: 1.0,
                image: self.img.clone(),
            },
        ));
    }
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Self {
        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.
        let wgpu_render_state = cc.wgpu_render_state.as_ref().unwrap();

        let runtime = Runtime {
            adapter: wgpu_render_state.adapter.clone(),
            device: wgpu_render_state.device.clone(),
            queue: wgpu_render_state.queue.clone(),
        };
        let engine = Engine { runtime: runtime };

        let main_image_render_resources = ui::main_image::MainImageRenderResources::create(
            &wgpu_render_state.device,
            wgpu_render_state.target_format,
        );
        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(main_image_render_resources);

        let img = Arc::new(pepe_core::image::Image::create_from_bytes(&engine.runtime));

        Self {
            engine: engine,
            img: img,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label("The triangle is being painted using ");
                        ui.hyperlink_to("WGPU", "https://wgpu.rs");
                        ui.label(" (Portable Rust graphics API awesomeness)");
                    });
                    ui.label("It's not a very impressive demo, but it shows you can embed 3D inside of egui.");

                    if ui.button("Select image file").clicked(){
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            let path_str = path.display();
                            println!("path {path_str}");
                        }
                    }

                    egui::Frame::canvas(ui.style()).show(ui, |ui| {
                        self.custom_painting(ui);
                    });
                });
        });
    }
}
