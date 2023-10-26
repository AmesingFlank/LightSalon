use crate::ui::{self, main_image::MainImageRenderResources, thumbnail::ThumbnailRenderResources};
use eframe::{
    egui::{self, accesskit::Vec2, CollapsingHeader, Ui, Visuals},
    emath::remap,
    epaint::Color32,
};
use egui_extras::{Column, TableBuilder};
use salon_core::{
    editor::EditorState,
    engine::{Engine, ImageHistogram},
    image::Image,
    ir::{AdjustExposureOp, Module, Op},
    library::AddImageResult,
    runtime::Runtime,
    session::Session,
};
use std::f64::consts::TAU;
use std::{num::NonZeroU64, sync::Arc};

use eframe::{
    egui_wgpu::wgpu::util::DeviceExt,
    egui_wgpu::{self, wgpu},
};

use egui_plot::{
    Arrows, AxisBools, AxisHints, Bar, BarChart, BoxElem, BoxPlot, BoxSpread, CoordinatesFormatter,
    Corner, GridInput, GridMark, HLine, Legend, Line, LineStyle, MarkerShape, Plot, PlotImage,
    PlotPoint, PlotPoints, PlotResponse, Points, Polygon, Text, VLine,
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

        let runtime = Arc::new(Runtime::new(
            wgpu_render_state.adapter.clone(),
            wgpu_render_state.device.clone(),
            wgpu_render_state.queue.clone(),
        ));

        let session = Session::new(runtime.clone());

        let mut renderer = wgpu_render_state.renderer.write();

        let main_image_render_resources =
            MainImageRenderResources::new(runtime.clone(), wgpu_render_state.target_format);
        renderer
            .callback_resources
            .insert(main_image_render_resources);

        let thumbnail_render_resources =
            ThumbnailRenderResources::new(runtime.clone(), wgpu_render_state.target_format);
        renderer
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
        if let Some(ref result) = self.session.current_process_result {
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
                self.ui_state.lowest_rendered_mip = Image::get_lowest_rendered_mip(
                    image.properties.dimensions,
                    (size.x as u32, size.y as u32),
                );
                ui.centered_and_justified(|ui| {
                    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::drag());
                    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                        rect,
                        ui::main_image::MainImageCallback {
                            image: image.clone(),
                        },
                    ));
                });
            }
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

    fn histogram(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Histogram")
            .default_open(true)
            .show(ui, |ui| {
                if let Some(ref result) = self.session.current_process_result {
                    if let Some(ref stats) = result.statistics {
                        let hist = &stats.histogram_final;

                        let get_line_data = |v: &Vec<u32>| {
                            let line_data: Vec<[f64; 2]> = (0..hist.num_bins)
                                .map(|i| [i as f64 / hist.num_bins as f64, v[i as usize] as f64])
                                .collect();
                            line_data
                        };

                        let r_line_data = get_line_data(&hist.r);
                        let g_line_data = get_line_data(&hist.g);
                        let b_line_data = get_line_data(&hist.b);
                        let luma_line_data = get_line_data(&hist.luma);

                        let r_line = Line::new(r_line_data)
                            .color(Color32::from_rgb(205, 25, 25))
                            .fill(0.0);
                        let g_line = Line::new(g_line_data)
                            .color(Color32::from_rgb(25, 205, 25))
                            .fill(0.0);
                        let b_line = Line::new(b_line_data)
                            .color(Color32::from_rgb(25, 25, 205))
                            .fill(0.0);
                        let luma_line = Line::new(luma_line_data)
                            .color(Color32::from_rgb(200, 200, 200))
                            .fill(0.0);

                        let img_dim = result.final_image.as_ref().unwrap().properties.dimensions;
                        let num_pixels = img_dim.0 * img_dim.1;
                        let mut y_top = hist.max_value() as f32;
                        y_top = y_top.min(10.0 * num_pixels as f32 / hist.num_bins as f32);

                        let plot = Plot::new("histogram")
                            .height(self.ui_state.last_frame_size.unwrap().1 * 0.1)
                            .show_x(false)
                            .show_y(false)
                            .allow_zoom(false)
                            .allow_scroll(false)
                            .allow_double_click_reset(false)
                            .allow_drag(false)
                            .include_x(0.0)
                            .include_x(1.0)
                            .include_y(0.0)
                            .include_y(y_top)
                            .show_axes([false, false])
                            .show_grid([false, false]);
                        plot.show(ui, |plot_ui| {
                            plot_ui.line(r_line);
                            plot_ui.line(g_line);
                            plot_ui.line(b_line);
                            plot_ui.line(luma_line);
                        });
                    }
                }
            });
    }

    fn curve(&mut self, ui: &mut Ui, editor_state: &mut EditorState) {
        CollapsingHeader::new("Curve")
            .default_open(true)
            .show(ui, |ui| {
                if let Some(ref result) = self.session.current_process_result {
                    if let Some(ref stats) = result.statistics {
                        let margin = 0.02;
                        let plot = Plot::new("curve")
                            .height(self.ui_state.last_frame_size.unwrap().1 * 0.2)
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
                        let mut control_points: Vec<[f64; 2]> = Vec::new();
                        for p in editor_state.curve_control_points.iter() {
                            control_points.push([p.0 as f64, p.1 as f64]);
                        }
                        let control_points = Points::new(control_points)
                            .shape(MarkerShape::Circle)
                            .radius(self.ui_state.last_frame_size.unwrap().1 * 0.003)
                            .filled(false)
                            .color(Color32::from_gray(255));
                        let response = plot.show(ui, |plot_ui| {
                            plot_ui.points(control_points);
                        });
                        if response.response.clicked() {
                            println!("clicked");
                        }
                    }
                }
            });
    }

    fn light_adjust(&mut self, ui: &mut Ui, editor_state: &mut EditorState) {
        CollapsingHeader::new("Light")
            .default_open(true)
            .show(ui, |ui| {
                ui.spacing_mut().slider_width = ui.available_width() * 0.6;
                ui.add(
                    egui::Slider::new(&mut editor_state.exposure_val, -4.0..=4.0).text("Exposure"),
                );

                ui.add(
                    egui::Slider::new(&mut editor_state.contrast_val, -100.0..=100.0)
                        .text("Contrast"),
                );

                ui.add(
                    egui::Slider::new(&mut editor_state.highlights_val, -100.0..=100.0)
                        .text("Highlights"),
                );

                ui.add(
                    egui::Slider::new(&mut editor_state.shadows_val, -100.0..=100.0)
                        .text("Shadows"),
                );
            });
    }

    fn color_adjust(&mut self, ui: &mut Ui, editor_state: &mut EditorState) {
        CollapsingHeader::new("Color")
            .default_open(true)
            .show(ui, |ui| {
                ui.spacing_mut().slider_width = ui.available_width() * 0.6;
                ui.add(
                    egui::Slider::new(&mut editor_state.temperature_val, -100.0..=100.0)
                        .text("Temperature"),
                );
                ui.add(egui::Slider::new(&mut editor_state.tint_val, -100.0..=100.0).text("Tint"));
                ui.add(
                    egui::Slider::new(&mut editor_state.vibrance_val, -100.0..=100.0)
                        .text("Vibrance"),
                );
                ui.add(
                    egui::Slider::new(&mut editor_state.saturation_val, -100.0..=100.0)
                        .text("Saturation"),
                );
            });
    }

    fn editor(&mut self, ui: &mut Ui) {
        let mut editor_state = self.session.editor.current_state.clone();
        self.histogram(ui);
        self.light_adjust(ui, &mut editor_state);
        self.color_adjust(ui, &mut editor_state);
        self.curve(ui, &mut editor_state);

        if self.session.current_image_index.is_none() {
            return;
        }
        if self.session.editor.current_state != editor_state {
            self.session.editor.current_state = editor_state;
            let module = self.session.editor.current_state.to_ir_module();
            let input_image_index = self.session.current_image_index.unwrap();
            let input_image = self.session.library.get_image(input_image_index);
            let result = self.session.engine.execute_module(&module, input_image);
            self.session.current_process_result = Some(result)
        }
    }

    fn get_visuals(&self) -> Visuals {
        Visuals {
            panel_fill: Color32::from_gray(32),
            ..Visuals::dark()
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.set_visuals(self.get_visuals());

        let is_first_frame = self.ui_state.last_frame_size.is_none();
        let last_frame_size = frame.info().window_info.size; // egui has a 1-frame delay
        self.ui_state.last_frame_size = Some((last_frame_size.x, last_frame_size.y));

        if is_first_frame {
            // if the screen is smaller than then window size we requested, then, on the first frame,
            // the frame size won't accurately reflection the actual frame size, so the sizing of side panels will be off
            return;
        }

        {
            let mut renderer = frame.wgpu_render_state().unwrap().renderer.write();

            let main_image_resources: &mut MainImageRenderResources =
                renderer.callback_resources.get_mut().unwrap();
            main_image_resources.reset();

            let thumbnail_resources: &mut ThumbnailRenderResources =
                renderer.callback_resources.get_mut().unwrap();
            thumbnail_resources.reset();
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
        egui::SidePanel::right("editor_panel")
            .default_width(last_frame_size.x * 0.2)
            .resizable(true)
            .show(ctx, |ui| {
                ui.set_width(ui.available_width());
                self.editor(ui);
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.main_image(ctx, frame, ui);
        });
    }
}

struct AppUiState {
    last_frame_size: Option<(f32, f32)>,
    lowest_rendered_mip: u32,
}

impl AppUiState {
    fn new() -> Self {
        AppUiState {
            last_frame_size: None,
            lowest_rendered_mip: 0,
        }
    }
}
