use crate::ui;
use eframe::egui::{self, Ui};
use pepe_core::{engine::Engine, library::AddImageResult, runtime::Runtime, session::Session};
use std::{num::NonZeroU64, sync::Arc};

use eframe::{
    egui_wgpu::wgpu::util::DeviceExt,
    egui_wgpu::{self, wgpu},
};

pub struct App {
    session: Session,
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

    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Self {
        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.
        let wgpu_render_state = cc.wgpu_render_state.as_ref().unwrap();

        let runtime = Runtime {
            adapter: wgpu_render_state.adapter.clone(),
            device: wgpu_render_state.device.clone(),
            queue: wgpu_render_state.queue.clone(),
        };

        let session = Session::new(Arc::new(runtime));

        let main_image_render_resources = ui::main_image::MainImageRenderResources::create(
            &wgpu_render_state.device,
            wgpu_render_state.target_format,
        );
        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(main_image_render_resources);

        Self { session }
    }
}

impl App {
    fn file_dialogue_import_image(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            let add_result = self.session.library.as_mut().add(path.to_str().unwrap());
            let selected_image: Option<u32> = match add_result {
                AddImageResult::AddedNewImage(i) => Some(i),
                AddImageResult::ImageAlreadyExists(i) => Some(i),
                AddImageResult::Error(_) => None,
            };
            match selected_image {
                Some(i) => {
                    let img = self.session.library.as_mut().get_image(i);
                    self.session.working_image_history.clear();
                    self.session.working_image_history.push(img);
                }
                None => {}
            };
        }
    }

    fn main_image(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame, ui: &mut Ui) {
        if self.session.working_image_history.len() > 0 {
            let max_x = ui.available_width();
            let max_y = ui.available_height();
            let ui_aspect_ratio = max_y / max_x;

            let image = self.session.working_image_history.last().unwrap().clone();
            let image_aspect_ratio = image.dimensions.1 as f32 / image.dimensions.0 as f32;

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
                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::drag());
                    ui.horizontal_centered(|ui| {
                        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                            rect,
                            ui::main_image::MainImageCallback {
                                arg: 1.0,
                                image: image,
                            },
                        ));
                    });
                });
            });
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let frame_size = frame.info().window_info.size;
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Import Image").clicked() {
                        ui.close_menu();
                        self.file_dialogue_import_image();
                    }
                });
            });
        });
        egui::SidePanel::left("library_panel")
            .default_width(frame_size.x * 0.2)
            .resizable(true)
            .show(ctx, |ui| {
                ui.set_width(ui.available_width());
            });
        egui::SidePanel::right("tools_panel")
            .default_width(frame_size.x * 0.2)
            .resizable(true)
            .show(ctx, |ui| {
                ui.set_width(ui.available_width());
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.main_image(ctx, frame, ui);
        });
    }
}
