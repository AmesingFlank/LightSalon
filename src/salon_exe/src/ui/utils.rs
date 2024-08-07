use std::ops::{Add, Mul, Sub};

use eframe::{
    egui::{self},
    epaint::Pos2,
};
use salon_core::{
    editor::Edit,
    library::Album,
    utils::{
        vec::{vec2, Vec2},
    },
};

use super::AppUiState;

pub fn pos2_to_vec2(p: Pos2) -> Vec2<f32> {
    vec2((p.x, p.y))
}

pub fn legalize_ui_state(ui_state: &mut AppUiState, edit: &Edit) {
    if ui_state.selected_mask_index >= edit.masked_edits.len() {
        ui_state.selected_mask_index = 0;
        ui_state.selected_mask_term_index = None;
    }
    if let Some(term) = ui_state.selected_mask_term_index {
        if term
            >= edit.masked_edits[ui_state.selected_mask_index]
                .mask
                .terms
                .len()
        {
            ui_state.selected_mask_term_index = None;
        }
    }
}

pub fn get_max_image_size(image_aspect_ratio: f32, max_width: f32, max_height: f32) -> egui::Vec2 {
    let ui_aspect_ratio = max_width / max_height;

    let size = if ui_aspect_ratio >= image_aspect_ratio {
        egui::Vec2 {
            x: max_height * image_aspect_ratio,
            y: max_height,
        }
    } else {
        egui::Vec2 {
            x: max_width,
            y: max_width / image_aspect_ratio,
        }
    };
    size
}

#[derive(Clone)]
pub struct AnimatedValue<T: Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T> + Copy> {
    pub from: T,
    pub to: T,
    pub duration: instant::Duration,
    start_time: instant::Instant,
}

impl<T: Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T> + Copy> AnimatedValue<T> {
    pub fn new(from: T, to: T, duration: instant::Duration) -> Self {
        Self {
            from,
            to,
            duration,
            start_time: instant::Instant::now(),
        }
    }

    pub fn from_constant(val: T) -> Self {
        Self::new(val, val, instant::Duration::from_secs_f32(0.0))
    }

    pub fn get(&self) -> T {
        let now = instant::Instant::now();
        let time_delta = (now - self.start_time).as_secs_f32();
        let ratio = time_delta / self.duration.as_secs_f32();
        let ratio = ratio.min(1.0);
        self.from + (self.to - self.from) * ratio
    }

    pub fn completed(&self) -> bool {
        let now = instant::Instant::now();
        let time_delta = (now - self.start_time).as_secs_f32();
        time_delta >= self.duration.as_secs_f32()
    }
}

pub fn get_album_name_text_with_emoji_and_count(album: &Album) -> String {
    "📷 ".to_owned() + album.name.as_str() + " (" + album.num_images().to_string().as_str() + ")"
}
