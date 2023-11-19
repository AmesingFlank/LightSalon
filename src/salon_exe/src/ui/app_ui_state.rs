use std::{fmt, time::SystemTime};

pub struct AppUiState {
    pub last_frame_size: Option<(f32, f32)>,
    pub fps_counter: FpsCounterState,

    pub show_grid: bool,

    pub selected_curve_control_point_index: Option<usize>,
    pub curve_scope: CurveScope,

    pub color_mixer_color_index: usize,
}

impl AppUiState {
    pub fn new() -> Self {
        AppUiState {
            last_frame_size: None,
            fps_counter: FpsCounterState::new(),
            show_grid: false,
            selected_curve_control_point_index: None,
            curve_scope: CurveScope::RGB,
            color_mixer_color_index: 0,
        }
    }
}

pub struct FpsCounterState {
    pub last_fps: f32,
    pub last_fps_record_time: SystemTime,
    pub frames_since_last_fps_record: u32,
}

impl FpsCounterState {
    pub fn new() -> Self {
        FpsCounterState {
            last_fps: 0.0,
            last_fps_record_time: SystemTime::now(),
            frames_since_last_fps_record: 0u32,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum CurveScope {
    RGB,
    R,
    G,
    B,
}

impl fmt::Display for CurveScope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}
