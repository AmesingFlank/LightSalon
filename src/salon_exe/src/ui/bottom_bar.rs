use eframe::{egui::{Ui, self}, egui_wgpu};
use egui_extras::{Column, TableBuilder};
use salon_core::session::Session;

use super::{AppUiState, ThumbnailCallback};

use std::time::{SystemTime, UNIX_EPOCH};


pub fn bottom_bar(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    let now = SystemTime::now();
    let duration_since_last_frame = now
        .duration_since(ui_state.last_frame_time)
        .expect("Time went backwards").as_secs_f32();
    let FPS = 1.0 / duration_since_last_frame;
    let FPS_msg = "FPS: ".to_owned() + FPS.to_string().as_str();
    ui_state.last_frame_time = now;
    ui.label(FPS_msg);
}