use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::{fmt, time::SystemTime};

use eframe::egui;
use salon_core::library::LibraryImageIdentifier;
use salon_core::runtime::{Image, Runtime, Toolbox};

use super::ImageImportDialog;

pub struct AppUiState {
    pub last_frame_size: Option<(f32, f32)>,
    pub fps_counter: FpsCounterState,

    pub show_grid: bool,
    pub show_comparison: bool,

    pub editor_panel: EditorPanel,

    pub selected_curve_control_point_index: Option<usize>,
    pub curve_scope: CurveScope,

    pub color_mixer_color_index: usize,

    pub crop_drag_state: CropDragState,

    pub selected_mask_index: usize,
    pub selected_mask_term_index: Option<usize>,
    pub mask_edit_state: MaskEditState,

    pub import_image_dialog: ImageImportDialog,
}

impl AppUiState {
    pub fn new(runtime: Arc<Runtime>, toolbox: Arc<Toolbox>, context: egui::Context) -> Self {
        AppUiState {
            last_frame_size: None,
            fps_counter: FpsCounterState::new(),
            show_grid: false,
            show_comparison: false,
            editor_panel: EditorPanel::LightAndColor,
            selected_curve_control_point_index: None,
            curve_scope: CurveScope::RGB,
            color_mixer_color_index: 0,
            crop_drag_state: CropDragState::new(),
            selected_mask_index: 0,
            selected_mask_term_index: None,
            mask_edit_state: MaskEditState::new(),
            import_image_dialog: ImageImportDialog::new(
                runtime.clone(),
                toolbox.clone(),
                context.clone(),
            ),
        }
    }

    pub fn reset_for_different_image(&mut self) {
        self.selected_curve_control_point_index = None;
        self.color_mixer_color_index = 0;
        self.selected_mask_index = 0;
        self.selected_mask_term_index = None;
        self.mask_edit_state.dragged_control_point_index = None;
    }
}

pub struct FpsCounterState {
    pub last_fps: f32,
    pub last_fps_record_time: instant::Instant,
    pub frames_since_last_fps_record: u32,
}

impl FpsCounterState {
    pub fn new() -> Self {
        FpsCounterState {
            last_fps: 0.0,
            last_fps_record_time: instant::Instant::now(),
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

#[derive(PartialEq)]
pub enum EditorPanel {
    LightAndColor,
    CropAndRotate,
}

pub struct CropDragState {
    pub edge_or_corner: Option<CropDragEdgeOrCorner>,
    pub translation: bool,
}

impl CropDragState {
    pub fn new() -> Self {
        Self {
            edge_or_corner: None,
            translation: false,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum CropDragEdgeOrCorner {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl CropDragEdgeOrCorner {
    pub fn has_left(&self) -> bool {
        match *self {
            CropDragEdgeOrCorner::Left
            | CropDragEdgeOrCorner::TopLeft
            | CropDragEdgeOrCorner::BottomLeft => true,
            _ => false,
        }
    }

    pub fn has_right(&self) -> bool {
        match *self {
            CropDragEdgeOrCorner::Right
            | CropDragEdgeOrCorner::TopRight
            | CropDragEdgeOrCorner::BottomRight => true,
            _ => false,
        }
    }

    pub fn has_top(&self) -> bool {
        match *self {
            CropDragEdgeOrCorner::Top
            | CropDragEdgeOrCorner::TopLeft
            | CropDragEdgeOrCorner::TopRight => true,
            _ => false,
        }
    }

    pub fn has_bottom(&self) -> bool {
        match *self {
            CropDragEdgeOrCorner::Bottom
            | CropDragEdgeOrCorner::BottomLeft
            | CropDragEdgeOrCorner::BottomRight => true,
            _ => false,
        }
    }
}

pub struct MaskEditState {
    pub dragged_control_point_index: Option<usize>,
}

impl MaskEditState {
    pub fn new() -> Self {
        Self {
            dragged_control_point_index: None,
        }
    }
}

pub enum AddedImage {
    Image(Arc<Image>),
    ImageFromPath(PathBuf),
}
