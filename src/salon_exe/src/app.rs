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
    #[cfg(not(target_arch = "wasm32"))]
    pub fn main() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .init();
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size(egui::vec2(1920.0, 1080.0)),
            renderer: eframe::Renderer::Wgpu,
            wgpu_options: egui_wgpu::WgpuConfiguration {
                device_descriptor: Arc::new(|_adapter| wgpu::DeviceDescriptor {
                    required_limits: Self::get_required_wgpu_limits(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        let _ = eframe::run_native(
            "Light Salon",
            options,
            Box::new(|_cc| Box::new(App::new(_cc))),
        );
    }

    // When compiling to web using trunk:
    #[cfg(target_arch = "wasm32")]
    pub fn main() {
        // Redirect `log` message to `console.log` and friends:
        eframe::WebLogger::init(log::LevelFilter::Debug).ok();

        let web_options = eframe::WebOptions {
            wgpu_options: egui_wgpu::WgpuConfiguration {
                supported_backends: wgpu::Backends::BROWSER_WEBGPU,
                device_descriptor: Arc::new(|_adapter| wgpu::DeviceDescriptor {
                    required_limits: Self::get_required_wgpu_limits(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        wasm_bindgen_futures::spawn_local(async {
            eframe::WebRunner::new()
                .start(
                    "the_canvas_id", // hardcode it
                    web_options,
                    Box::new(|_cc| Box::new(App::new(_cc))),
                )
                .await
                .expect("failed to start eframe");
        });
    }

    fn get_required_wgpu_limits() -> wgpu::Limits {
        // 100MP medium format digital sensor file size: 11656 x 8742
        let max_dim = 11656;
        let max_buff_size = 11656 * 8742 * 4;
        wgpu::Limits {
            max_texture_dimension_1d: max_dim,
            max_texture_dimension_2d: max_dim,
            max_buffer_size: max_buff_size,
            max_storage_buffer_binding_size: max_buff_size as u32,
            ..Default::default()
        }
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
        App::create_widget_render_resources(wgpu_render_state, runtime.clone());

        Self {
            session,
            ui_state: AppUiState::new(runtime.clone(), cc.egui_ctx.clone()),
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
        let last_frame_size = ctx.input(|i| i.screen_rect.size()); // egui has a 1-frame delay
        self.ui_state.last_frame_size = Some((last_frame_size.x, last_frame_size.y));

        if is_first_frame {
            // if the screen is smaller than then window size we requested, then, on the first frame,
            // the frame size won't accurately reflection the actual frame size, so the sizing of side panels will be off
            return;
        }

        self.reset_widget_render_resources(frame);
        ui::app_ui(ctx, &mut self.session, &mut self.ui_state);

        if let Some(added_image) = self.ui_state.import_image_dialog.get_added_image() {
            let index = self.session.library.add_image(added_image.image);
            self.ui_state.reset_for_different_image();
            self.session.set_current_image(index);
        }
    }
}
