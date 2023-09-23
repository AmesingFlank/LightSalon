use crate::ui;
use eframe::egui::{self, accesskit::Vec2, Ui};
use egui_extras::{Column, TableBuilder};
use salon_core::{engine::Engine, library::AddImageResult, runtime::Runtime, session::Session};
use std::{num::NonZeroU64, sync::Arc};

use eframe::{
    egui_wgpu::wgpu::util::DeviceExt,
    egui_wgpu::{self, wgpu},
};

pub struct App {
    session: Session,
    ui_state: AppUiState,
}

impl App {
    pub fn main() {
        env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(1920.0, 1080.0)),
            renderer: eframe::Renderer::Wgpu,
            ..Default::default()
        };
        let _ = eframe::run_native(
            "Light Salon",
            options,
            Box::new(|_cc| Box::new(App::new(_cc))),
        );
    }

    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Self {
        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.
        let wgpu_render_state = cc.wgpu_render_state.as_ref().unwrap();

        let runtime = Arc::new(Runtime {
            adapter: wgpu_render_state.adapter.clone(),
            device: wgpu_render_state.device.clone(),
            queue: wgpu_render_state.queue.clone(),
        });

        let session = Session::new(runtime.clone());

        let main_image_render_resources = ui::main_image::MainImageRenderResources::create(
            runtime.as_ref(),
            wgpu_render_state.target_format,
        );
        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(main_image_render_resources);

        let thumbnail_render_resources = ui::thumbnail::ThumbnailRenderResources::create(
            runtime.as_ref(),
            wgpu_render_state.target_format,
        );
        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(thumbnail_render_resources);

        Self {
            session,
            ui_state: AppUiState::new(),
        }
    }
}

impl App {
    fn file_dialogue_import_image(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            let add_result = self.session.library.as_mut().add(path.to_str().unwrap());
            let selected_image: Option<usize> = match add_result {
                AddImageResult::AddedNewImage(i) => Some(i),
                AddImageResult::ImageAlreadyExists(i) => Some(i),
                AddImageResult::Error(_) => None,
            };
            match selected_image {
                Some(i) => self.session.set_current_image(i),
                None => {}
            };
        }
    }

    fn main_image(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame, ui: &mut Ui) {
        if self.session.working_image_history.len() > 0 {
            let max_x = ui.available_width();
            let max_y = ui.available_height();
            let ui_aspect_ratio = max_y / max_x;

            let image = self.session.working_image_history.last().unwrap().clone();
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
                    ui::main_image::MainImageCallback { image: image },
                ));
            });
        }
    }

    fn image_library(&mut self, ui: &mut Ui) {
        let mut table = TableBuilder::new(ui).column(Column::auto()).cell_layout(
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
        );
        let row_height = self.ui_state.last_frame_size.unwrap().1 * 0.1;
        let image_height = row_height * 0.8;
        table.body(|mut body| {
            body.rows(
                row_height,
                self.session.library.num_images() as usize,
                |row_index, mut row| {
                    row.col(|ui| {
                        let image = self.session.library.get_image(row_index);
                        let aspect_ratio = image.aspect_ratio();
                        let image_width = image_height / aspect_ratio;
                        let size = egui::Vec2 {
                            x: image_width,
                            y: image_height,
                        };
                        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
                        ui.centered_and_justified(|ui| {
                            ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                                rect,
                                ui::thumbnail::ThumbnailCallback { image: image },
                            ));
                        });
                        if response.clicked() {
                            self.session.set_current_image(row_index);
                        }
                    });
                },
            );
        });
    }

    fn tools(&mut self, ui: &mut Ui) {
        {
            let old_exposure = self.session.exposure_val.clone();
            ui.add(egui::Slider::new(&mut self.session.exposure_val, 0.0..=100.0).text("Exposure"));
            if old_exposure != self.session.exposure_val {
                if self.session.working_image_history.len() > 0 {
                    if self.session.working_image_history.len() == 1 {
                        let dimensions = self.session.working_image_history[0].dimensions;
                        let output = self.session.engine.runtime.create_image_of_size(dimensions);
                        self.session.working_image_history.push(Arc::new(output));
                    }
                    let input = self.session.working_image_history[0].as_ref();
                    let output = self.session.working_image_history[1].as_ref();
                    self.session.engine.exposure_op.apply(
                        input,
                        output,
                        old_exposure,
                        self.session.exposure_val,
                    );
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let is_first_frame = self.ui_state.last_frame_size.is_none();
        let last_frame_size = frame.info().window_info.size; // egui has a 1-frame delay
        self.ui_state.last_frame_size = Some((last_frame_size.x, last_frame_size.y));

        if is_first_frame {
            // if the screen is smaller than then window size we requested, then, on the first frame,
            // the frame size won't accurately reflection the actual frame size, so the sizing of side panels will be off
            return;
        }

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
            .default_width(last_frame_size.x * 0.2)
            .resizable(true)
            .show(ctx, |ui| {
                // ui.set_width(ui.available_width());
                self.image_library(ui);
            });
        egui::SidePanel::right("tools_panel")
            .default_width(last_frame_size.x * 0.2)
            .resizable(true)
            .show(ctx, |ui| {
                ui.set_width(ui.available_width());
                self.tools(ui);
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.main_image(ctx, frame, ui);
        });
    }
}

struct AppUiState {
    last_frame_size: Option<(f32, f32)>,
}

impl AppUiState {
    fn new() -> Self {
        AppUiState {
            last_frame_size: None,
        }
    }
}
