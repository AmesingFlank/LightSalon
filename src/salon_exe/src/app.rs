use crate::ui::{
    self, file_menu,
    widgets::{
        EditorSliderRectRenderResources, MainImageRenderResources, MaskIndicatorRenderResources,
        ThumbnailRenderResources,
    },
    AppUiState,
};
use eframe::{
    egui::{self, Visuals},
    egui_wgpu,
    epaint::Color32,
};
use salon_core::{runtime::Runtime, session::Session};

use std::sync::Arc;

pub struct App {
    session: Session,
    ui_state: AppUiState,
}

impl App {
    pub fn main() {
        env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size(egui::vec2(1920.0, 1080.0)),
            renderer: eframe::Renderer::Wgpu,
            ..Default::default()
        };
        let _ = eframe::run_native(
            "Light Salon",
            options,
            Box::new(|_cc| Box::new(App::new(_cc))),
        );
    }

    pub fn new(cc: &eframe::CreationContext) -> Self {
        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.
        let wgpu_render_state = cc.wgpu_render_state.as_ref().unwrap();

        let runtime = Arc::new(Runtime::new(
            wgpu_render_state.adapter.clone(),
            wgpu_render_state.device.clone(),
            wgpu_render_state.queue.clone(),
        ));

        let session = Session::new(runtime.clone());
        App::create_widget_render_resources(wgpu_render_state, runtime);

        Self {
            session,
            ui_state: AppUiState::new(),
        }
    }

    fn create_widget_render_resources(
        wgpu_render_state: &egui_wgpu::RenderState,
        runtime: Arc<Runtime>,
    ) {
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

        let resources =
            MaskIndicatorRenderResources::new(runtime.clone(), wgpu_render_state.target_format);
        renderer.callback_resources.insert(resources);
    }

    fn reset_widget_render_resources(&mut self, frame: &mut eframe::Frame) {
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

        let resources: &mut MaskIndicatorRenderResources =
            renderer.callback_resources.get_mut().unwrap();
        resources.reset();
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
        let last_frame_size = ctx.input(|i| i.viewport().inner_rect.unwrap().size()); // egui has a 1-frame delay
        self.ui_state.last_frame_size = Some((last_frame_size.x, last_frame_size.y));

        if is_first_frame {
            // if the screen is smaller than then window size we requested, then, on the first frame,
            // the frame size won't accurately reflection the actual frame size, so the sizing of side panels will be off
            return;
        }

        self.reset_widget_render_resources(frame);
        ui::app_ui(ctx, &mut self.session, &mut self.ui_state);

        //egui::Context::request_repaint(ctx);
    }
}
