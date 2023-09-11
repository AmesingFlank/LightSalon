mod app;

use app::App;

use std::borrow::Cow;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};


fn main() {
    let mut app = App{};

    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    {
        env_logger::init();
        pollster::block_on(app.run(event_loop, window));
    }
}
