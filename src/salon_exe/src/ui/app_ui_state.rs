use std::time::SystemTime;

pub struct AppUiState {
    pub last_frame_size: Option<(f32, f32)>,
    pub fps_counter: FpsCounterState,
    pub selected_curve_control_point_index: Option<usize>,
}

impl AppUiState {
    pub fn new() -> Self {
        AppUiState {
            last_frame_size: None,
            fps_counter: FpsCounterState::new(),
            selected_curve_control_point_index: None,
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
