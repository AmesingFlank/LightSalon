use eframe::egui;
use pepe_core::{engine::Engine, runtime::Runtime};
use std::num::NonZeroU64;
use crate::ui;

use eframe::{
    egui_wgpu::wgpu::util::DeviceExt,
    egui_wgpu::{self, wgpu},
};

pub struct App {
    angle: f32,
    engine: Engine,
}

impl App {
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

        Self {
            angle: 0.0,
            engine: engine,
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

                    egui::Frame::canvas(ui.style()).show(ui, |ui| {
                        self.custom_painting(ui);
                    });
                    ui.label("Drag to rotate!");
                });
        });
    }
}

impl App {
    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(300.0), egui::Sense::drag());

        self.angle += response.drag_delta().x * 0.01;
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            ui::main_image::MainImageCallback { angle: self.angle },
        ));
    }
}
