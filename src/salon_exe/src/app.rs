use crate::ui::widgets::{ImageFramingRenderResources, ImageGeometryEditRenderResources};
use crate::ui::{
    self, file_menu,
    widgets::{
        EditorSliderRectRenderResources, MainImageRenderResources, MaskIndicatorRenderResources,
        ThumbnailRenderResources,
    },
    AppUiState,
};
use crate::ui::{ui_set_current_editor_image, AddedImageOrAlbum};
use eframe::egui::style::Selection;
use eframe::{
    egui::{self, Visuals},
    egui_wgpu,
    epaint::Color32,
};
use salon_core::library::LibraryImageMetaData;
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
                    required_limits: Runtime::get_required_wgpu_limits(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        let _ = eframe::run_native(
            "Light Salon",
            options,
            Box::new(|_cc| Ok(Box::new(App::new(_cc)))),
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
                    required_limits: Runtime::get_required_wgpu_limits(),
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
                    Box::new(|_cc| Ok(Box::new(App::new(_cc)))),
                )
                .await
                .expect("failed to start eframe");
        });
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

        let mut session = Session::new(runtime.clone());
        let toolbox = session.toolbox.clone();
        App::create_widget_render_resources(wgpu_render_state, runtime.clone());

        Self {
            session,
            ui_state: AppUiState::new(runtime.clone(), toolbox),
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
            ImageGeometryEditRenderResources::new(runtime.clone(), wgpu_render_state.target_format);
        renderer.callback_resources.insert(resources);

        let resources =
            ImageFramingRenderResources::new(runtime.clone(), wgpu_render_state.target_format);
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

        let resources: &mut ImageGeometryEditRenderResources =
            renderer.callback_resources.get_mut().unwrap();
        resources.reset();

        let resources: &mut ImageFramingRenderResources =
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
            override_text_color: Some(egui::Color32::from_gray(255)),
            selection: Selection {
                bg_fill: Color32::from_gray(90),
                stroke: egui::Stroke::new(1.0, Color32::from_gray(255)),
            },
            ..Visuals::dark()
        }
    }

    fn maybe_handled_imported_image(&mut self, ctx: &egui::Context) {
        while let Some(added_image) = self.ui_state.import_image_dialog.get_added_image() {
            match added_image {
                AddedImageOrAlbum::ImagesFromPaths(paths) => {
                    if !paths.is_empty() {
                        let identifiers = self.session.library.add_items_from_paths(paths, None);
                        ui_set_current_editor_image(
                            &mut self.session,
                            &mut self.ui_state,
                            identifiers.last().unwrap().clone(),
                        );
                    }
                }
                AddedImageOrAlbum::Image(image, metadata) => {
                    let identifier = self.session.library.add_image_temp(image, None, metadata);
                    ui_set_current_editor_image(&mut self.session, &mut self.ui_state, identifier);
                }
                AddedImageOrAlbum::AlbumFromPath(album_dir) => {
                    let album_index = self.session.library.add_album_from_directory(album_dir);
                }
            }
        }
    }

    fn maybe_handle_dropped_image(&mut self, ctx: &egui::Context) {
        let raw_input = ctx.input(|i| i.raw.clone());
        for dropped_file in raw_input.dropped_files {
            if let Some(pathbuf) = dropped_file.path {
                let identifier = self
                    .session
                    .library
                    .add_single_item_from_path(pathbuf, None);
                ui_set_current_editor_image(&mut self.session, &mut self.ui_state, identifier);
            } else {
                if let Some(bytes) = dropped_file.bytes {
                    let file_name = dropped_file.name;
                    let file_name_parts: Vec<&str> = file_name.split(".").collect();
                    let ext = file_name_parts.last().unwrap().to_owned();

                    let image = self
                        .session
                        .runtime
                        .create_image_from_bytes_and_extension(bytes.as_ref(), ext);
                    match image {
                        Ok(img) => {
                            let metadata = LibraryImageMetaData {
                                name: Some(file_name),
                            };
                            let identifier =
                                self.session
                                    .library
                                    .add_image_temp(Arc::new(img), None, metadata);
                            ui_set_current_editor_image(
                                &mut self.session,
                                &mut self.ui_state,
                                identifier,
                            );
                        }
                        Err(_) => {}
                    }
                }
            }
        }
    }

    pub fn maybe_handle_app_exit(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.viewport().close_requested()) {
            self.session.on_exit();
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
            ctx.request_repaint();
            return;
        }

        self.reset_widget_render_resources(frame);
        ui::app_ui(ctx, &mut self.session, &mut self.ui_state);

        self.maybe_handle_dropped_image(ctx);
        self.maybe_handled_imported_image(ctx);
        self.maybe_handle_app_exit(ctx);

        self.session.library.poll_updates();

        ctx.request_repaint();
    }
}
