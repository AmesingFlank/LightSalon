use pepe_core::{engine::Engine, runtime::Runtime};
use std::{borrow::Cow, time::Instant};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub struct App {
    event_loop: EventLoop<()>,
    window: Window,
    engine: pepe_core::engine::Engine,
}

impl App {
    pub async fn new() -> App {
        let event_loop = EventLoop::new();
        let window = winit::window::Window::new(&event_loop).unwrap();
        let engine = Engine {
            runtime: Runtime::create_with_native_window(&window).await,
        };
        App {
            engine: engine,
            event_loop: event_loop,
            window: window,
        }
    }

    pub fn main(self) {
        env_logger::init();
        pollster::block_on(self.run());
    }

    pub async fn run(self) {
        let size = self.window.inner_size();
        // Load the shaders from disk
        let shader =
            self.engine
                .runtime
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
                });

        let pipeline_layout =
            self.engine
                .runtime
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let swapchain_capabilities = self
            .engine
            .runtime
            .surface
            .as_ref()
            .unwrap()
            .get_capabilities(&self.engine.runtime.adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let render_pipeline =
            self.engine
                .runtime
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(swapchain_format.into())],
                    }),
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        let mut config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        self.engine
            .runtime
            .surface
            .as_ref()
            .unwrap()
            .configure(&self.engine.runtime.device, &config);

        let mut imgui = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            &self.window,
            imgui_winit_support::HiDpiMode::Default,
        );
        imgui.set_ini_filename(None);

        let hidpi_factor = self.window.scale_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
            }]);
        //
        // Set up dear imgui wgpu renderer
        //
        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };

        let renderer_config = imgui_wgpu::RendererConfig {
            texture_format: swapchain_format,
            ..Default::default()
        };

        let mut renderer = imgui_wgpu::Renderer::new(
            &mut imgui,
            &self.engine.runtime.device,
            &self.engine.runtime.queue,
            renderer_config,
        );

        let mut last_frame = Instant::now();
        let mut last_cursor = None;

        self.event_loop.run(move |event, _, control_flow| {
            // Have the closure take ownership of the resources.
            // `event_loop.run` never returns, therefore we must do this to ensure
            // the resources are properly cleaned up.
            let _ = (
                &self.engine.runtime.instance,
                &self.engine.runtime.adapter,
                &shader,
                &pipeline_layout,
            );

            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    // Reconfigure the surface with the new size
                    config.width = size.width;
                    config.height = size.height;
                    self.engine
                        .runtime
                        .surface
                        .as_ref()
                        .unwrap()
                        .configure(&self.engine.runtime.device, &config);
                    // On macos the window needs to be redrawn manually after resizing
                    self.window.request_redraw();
                }
                Event::RedrawEventsCleared => {
                    let delta_s = last_frame.elapsed();
                    let now = Instant::now();
                    imgui.io_mut().update_delta_time(now - last_frame);
                    last_frame = now;
                    platform
                        .prepare_frame(imgui.io_mut(), &self.window)
                        .expect("Failed to prepare frame");
                    let ui = imgui.frame();

                    {
                        let window = ui.window("Hello world");
                        window
                            .size([300.0, 100.0], imgui::Condition::FirstUseEver)
                            .build(|| {
                                ui.text("Hello world!");
                                ui.text("This...is...imgui-rs on WGPU!");
                                ui.separator();
                                let mouse_pos = ui.io().mouse_pos;
                                ui.text(format!(
                                    "Mouse Position: ({:.1},{:.1})",
                                    mouse_pos[0], mouse_pos[1]
                                ));
                            });

                        let window = ui.window("Hello too");
                        window
                            .size([400.0, 200.0], imgui::Condition::FirstUseEver)
                            .position([400.0, 200.0], imgui::Condition::FirstUseEver)
                            .build(|| {
                                ui.text(format!("Frametime: {delta_s:?}"));
                            });
                        let mut demo_open = true;
                        ui.show_demo_window(&mut demo_open);
                    }

                    let frame = self
                        .engine
                        .runtime
                        .surface
                        .as_ref()
                        .unwrap()
                        .get_current_texture()
                        .expect("Failed to acquire next swap chain texture");
                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder =
                        self.engine.runtime.device.create_command_encoder(
                            &wgpu::CommandEncoderDescriptor { label: None },
                        );
                    {
                        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: None,
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                    store: true,
                                },
                            })],
                            depth_stencil_attachment: None,
                        });
                        rpass.set_pipeline(&render_pipeline);
                        rpass.draw(0..3, 0..1);

                        if last_cursor != Some(ui.mouse_cursor()) {
                            last_cursor = Some(ui.mouse_cursor());
                            platform.prepare_render(ui, &self.window);
                        }

                        renderer
                            .render(
                                imgui.render(),
                                &self.engine.runtime.queue,
                                &self.engine.runtime.device,
                                &mut rpass,
                            )
                            .expect("Rendering failed");
                    }

                    self.engine.runtime.queue.submit(Some(encoder.finish()));
                    frame.present();
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {}
            }

            platform.handle_event(imgui.io_mut(), &self.window, &event);
        });
    }
}
