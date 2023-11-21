use crate::ui::{
    self, file_menu,
    widgets::{
        EditorSliderRectRenderResources, MainImageRenderResources, ThumbnailRenderResources,
    },
    AppUiState,
};
use eframe::{
    egui::{self, accesskit::Vec2, CollapsingHeader, Ui, Visuals},
    emath::remap,
    epaint::Color32,
};
use egui_extras::{Column, TableBuilder};
use salon_core::{
    editor::GlobalEdit,
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

        let resources =
            MainImageRenderResources::new(runtime.clone(), wgpu_render_state.target_format);
        renderer.callback_resources.insert(resources);

        let resources =
            ThumbnailRenderResources::new(runtime.clone(), wgpu_render_state.target_format);
        renderer.callback_resources.insert(resources);

        let resources =
            EditorSliderRectRenderResources::new(runtime.clone(), wgpu_render_state.target_format);
        renderer.callback_resources.insert(resources);

        Self {
            session,
            ui_state: AppUiState::new(),
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

            let resources: &mut MainImageRenderResources =
                renderer.callback_resources.get_mut().unwrap();
            resources.reset();

            let resources: &mut ThumbnailRenderResources =
                renderer.callback_resources.get_mut().unwrap();
            resources.reset();

            let resources: &mut EditorSliderRectRenderResources =
                renderer.callback_resources.get_mut().unwrap();
            resources.reset();
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                file_menu(ui, &mut self.session);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    ui.checkbox(&mut self.ui_state.show_grid, "Show Grid");
                })
            });
        });
        egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
            ui::bottom_bar(ui, &mut self.session, &mut self.ui_state);
        });
        egui::SidePanel::left("library_panel")
            .default_width(last_frame_size.x * 0.2)
            .resizable(true)
            .show(ctx, |ui| {
                // ui.set_width(ui.available_width());
                ui::image_library(ui, &mut self.session, &mut self.ui_state);
            });
        egui::SidePanel::right("editor_panel")
            .default_width(last_frame_size.x * 0.2)
            .resizable(true)
            .show(ctx, |ui| {
                ui.set_width(ui.available_width());
                ui::editor(ui, &mut self.session, &mut self.ui_state);
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui::main_image(ctx, ui, &mut self.session, &mut self.ui_state);
        });
        //egui::Context::request_repaint(ctx);
    }
}
