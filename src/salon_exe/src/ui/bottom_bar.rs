use eframe::{
    egui::{Ui},
};

use salon_core::session::Session;

use super::AppUiState;



pub fn bottom_bar(ui: &mut Ui, _session: &mut Session, ui_state: &mut AppUiState) {
    let fps_counter = &mut ui_state.fps_counter;
    let now = instant::Instant::now();
    let duration_since_last_record = now
        .duration_since(fps_counter.last_fps_record_time)
        .as_secs_f32();
    let mut fps = fps_counter.last_fps;
    fps_counter.frames_since_last_fps_record += 1u32;
    if duration_since_last_record > 1.0 {
        fps = fps_counter.frames_since_last_fps_record as f32 / duration_since_last_record;
        fps_counter.last_fps_record_time = now;
        fps_counter.last_fps = fps;
        fps_counter.frames_since_last_fps_record = 0u32;
    }
    let fps_msg = "FPS: ".to_owned() + fps.to_string().as_str();
    ui.label(fps_msg);
}
