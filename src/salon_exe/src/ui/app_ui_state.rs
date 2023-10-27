
pub struct AppUiState {
    pub last_frame_size: Option<(f32, f32)>,

    pub selected_curve_control_point_index: Option<usize>,
}

impl AppUiState {
    pub fn new() -> Self {
        AppUiState {
            last_frame_size: None,
            selected_curve_control_point_index: None,
        }
    }
}
