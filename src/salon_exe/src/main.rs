mod app;
mod ui;

use eframe::egui;


use eframe::{
    egui_wgpu::{self},
};

fn main() {
    app::App::main();
}
