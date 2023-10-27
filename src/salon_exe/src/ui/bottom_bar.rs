use eframe::{egui::{Ui, self}, egui_wgpu};
use egui_extras::{Column, TableBuilder};
use salon_core::session::Session;

use super::{AppUiState, ThumbnailCallback};

use std::time::{SystemTime, UNIX_EPOCH};


pub fn bottom_bar(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    let now = SystemTime::now();
    let duration_since_last_record = now
        .duration_since(ui_state.last_fps_record_time)
        .expect("Time went backwards").as_secs_f32();
    let mut fps = ui_state.last_fps;
    ui_state.frames_since_last_fps_record += 1u32;
    if duration_since_last_record > 1.0 {
        fps = ui_state.frames_since_last_fps_record as f32 / duration_since_last_record;
        ui_state.last_fps_record_time = now;
        ui_state.last_fps = fps;
        ui_state.frames_since_last_fps_record = 0u32;
    }
    let fps_msg = "FPS: ".to_owned() + fps.to_string().as_str();
    ui.label(fps_msg);
}