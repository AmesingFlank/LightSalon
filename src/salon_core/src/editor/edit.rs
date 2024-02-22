use crate::ir::{ColorMixGroup, GlobalMask, Mask, MaskPrimitive, MaskTerm};

use crate::utils::rectangle::Rectangle;

use serde;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Edit {
    pub scale_factor: Option<f32>,
    pub crop: Option<Rectangle>,
    pub masked_edits: Vec<MaskedEdit>,
}

impl Edit {
    pub fn trivial() -> Self {
        Self {
            scale_factor: None,
            crop: None,
            masked_edits: vec![MaskedEdit::new(
                Mask {
                    terms: vec![MaskTerm {
                        primitive: MaskPrimitive::Global(GlobalMask::default()),
                        inverted: false,
                        subtracted: false,
                    }],
                },
                GlobalEdit::new(),
                "Global".to_string(),
            )],
        }
    }
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct MaskedEdit {
    pub mask: Mask,
    pub edit: GlobalEdit,
    pub name: String,
}

impl MaskedEdit {
    pub fn new(mask: Mask, edit: GlobalEdit, name: String) -> Self {
        Self { mask, edit, name }
    }
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct GlobalEdit {
    pub exposure: f32,
    pub contrast: f32,
    pub highlights: f32,
    pub shadows: f32,

    pub curve_control_points_all: Vec<(f32, f32)>,
    pub curve_control_points_r: Vec<(f32, f32)>,
    pub curve_control_points_g: Vec<(f32, f32)>,
    pub curve_control_points_b: Vec<(f32, f32)>,

    pub temperature: f32,
    pub tint: f32,
    pub vibrance: f32,
    pub saturation: f32,

    pub color_mixer_edits: [ColorMixGroup; 8],

    pub dehaze: f32,
    pub vignette: f32,
}

impl GlobalEdit {
    pub fn new() -> Self {
        GlobalEdit {
            exposure: 0.0,
            contrast: 0.0,
            highlights: 0.0,
            shadows: 0.0,

            curve_control_points_all: GlobalEdit::initial_control_points(),
            curve_control_points_r: GlobalEdit::initial_control_points(),
            curve_control_points_g: GlobalEdit::initial_control_points(),
            curve_control_points_b: GlobalEdit::initial_control_points(),

            temperature: 0.0,
            tint: 0.0,
            vibrance: 0.0,
            saturation: 0.0,

            color_mixer_edits: [ColorMixGroup::new(); 8],

            dehaze: 0.0,
            vignette: 0.0,
        }
    }

    pub fn initial_control_points() -> Vec<(f32, f32)> {
        vec![(0.0, 0.0), (1.0, 1.0)]
    }
}
