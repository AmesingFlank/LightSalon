use std::time::{SystemTime};

pub struct AppUiState {
    pub last_frame_size: Option<(f32, f32)>,

    pub last_frame_time: SystemTime,

    pub selected_curve_control_point_index: Option<usize>,
}

impl AppUiState {
    pub fn new() -> Self {
        AppUiState {
            last_frame_size: None,
            last_frame_time: SystemTime::now(),
            selected_curve_control_point_index: None,
        }
    }
}
