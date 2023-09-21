mod app;
mod ui;

use eframe::egui;
use std::num::NonZeroU64;

use eframe::{
    egui_wgpu::wgpu::util::DeviceExt,
    egui_wgpu::{self, wgpu},
};

fn main() {
    app::App::main();
}
