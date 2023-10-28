use crate::ir::{
    AdjustContrastOp, AdjustExposureOp, AdjustHighlightsAndShadowsOp, AdjustTemperatureAndTintOp,
    AdjustVibranceAndSaturationOp, ComputeBasicStatisticsOp, Id, Module, Op, ApplyCurveOp,
};

pub struct Editor {
    pub current_state: EditorState,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            current_state: EditorState::new(),
        }
    }

    pub fn reset_state(&mut self) {
        self.current_state = EditorState::new();
    }
}

#[derive(Clone, PartialEq)]
pub struct EditorState {
    pub exposure_val: f32,
    pub contrast_val: f32,
    pub highlights_val: f32,
    pub shadows_val: f32,
    pub temperature_val: f32,
    pub tint_val: f32,
    pub vibrance_val: f32,
    pub saturation_val: f32,

    pub curve_control_points: Vec<(f32, f32)>,
}

impl EditorState {
    pub fn new() -> Self {
        EditorState {
            exposure_val: 0.0,
            contrast_val: 0.0,
            highlights_val: 0.0,
            shadows_val: 0.0,
            temperature_val: 0.0,
            tint_val: 0.0,
            vibrance_val: 0.0,
            saturation_val: 0.0,
            curve_control_points: EditorState::initial_control_points(),
        }
    }
    pub fn to_ir_module(&self) -> Module {
        let mut module = Module::new_trivial();

        let mut current_output_id = module.get_output_id().expect("expecting an output id");

        self.maybe_add_exposure(&mut module, &mut current_output_id);
        self.maybe_add_contrast(&mut module, &mut current_output_id);
        self.maybe_add_highlights_shadows(&mut module, &mut current_output_id);
        self.maybe_add_temperature_tint(&mut module, &mut current_output_id);
        self.maybe_add_vibrance_saturation(&mut module, &mut current_output_id);
        self.maybe_add_curve(&mut module, &mut current_output_id);

        module.add_data_for_editor_ops();

        module
    }

    fn maybe_add_exposure(&self, module: &mut Module, current_output_id: &mut Id) {
        if self.exposure_val != 0.0 {
            let exposure_adjusted_image_id = module.alloc_id();
            module.push_op(Op::AdjustExposure(AdjustExposureOp {
                result: exposure_adjusted_image_id,
                arg: *current_output_id,
                exposure: self.exposure_val,
            }));
            module.set_output_id(exposure_adjusted_image_id);
            *current_output_id = exposure_adjusted_image_id;
        }
    }

    fn maybe_add_contrast(&self, module: &mut Module, current_output_id: &mut Id) {
        if self.contrast_val != 0.0 {
            let basic_stats_id = module.alloc_id();
            module.push_op(Op::ComputeBasicStatistics(ComputeBasicStatisticsOp {
                result: basic_stats_id,
                arg: *current_output_id,
            }));

            let contrast_adjusted_image_id = module.alloc_id();
            module.push_op(Op::AdjustContrast(AdjustContrastOp {
                result: contrast_adjusted_image_id,
                arg: *current_output_id,
                basic_stats: basic_stats_id,
                contrast: self.contrast_val,
            }));
            module.set_output_id(contrast_adjusted_image_id);
            *current_output_id = contrast_adjusted_image_id;
        }
    }

    fn maybe_add_highlights_shadows(&self, module: &mut Module, current_output_id: &mut Id) {
        if self.highlights_val != 0.0 || self.shadows_val != 0.0 {
            let adjusted_image_id = module.alloc_id();
            module.push_op(Op::AdjustHighlightsAndShadows(
                AdjustHighlightsAndShadowsOp {
                    result: adjusted_image_id,
                    arg: *current_output_id,
                    highlights: self.highlights_val,
                    shadows: self.shadows_val,
                },
            ));
            module.set_output_id(adjusted_image_id);
            *current_output_id = adjusted_image_id;
        }
    }

    fn maybe_add_temperature_tint(&self, module: &mut Module, current_output_id: &mut Id) {
        if self.temperature_val != 0.0 || self.tint_val != 0.0 {
            let temperature_tint_adjusted_image_id = module.alloc_id();
            module.push_op(Op::AdjustTemperatureAndTint(AdjustTemperatureAndTintOp {
                result: temperature_tint_adjusted_image_id,
                arg: *current_output_id,
                temperature: self.temperature_val,
                tint: self.tint_val,
            }));
            module.set_output_id(temperature_tint_adjusted_image_id);
            *current_output_id = temperature_tint_adjusted_image_id;
        }
    }

    fn maybe_add_vibrance_saturation(&self, module: &mut Module, current_output_id: &mut Id) {
        if self.vibrance_val != 0.0 || self.saturation_val != 0.0 {
            let adjusted_image_id = module.alloc_id();
            module.push_op(Op::AdjustVibranceAndSaturation(
                AdjustVibranceAndSaturationOp {
                    result: adjusted_image_id,
                    arg: *current_output_id,
                    vibrance: self.vibrance_val,
                    saturation: self.saturation_val,
                },
            ));
            module.set_output_id(adjusted_image_id);
            *current_output_id = adjusted_image_id;
        }
    }

    fn maybe_add_curve(&self, module: &mut Module, current_output_id: &mut Id) {
        if self.curve_control_points != Self::initial_control_points() {
            let adjusted_image_id = module.alloc_id();
            module.push_op(Op::ApplyCurve(
                ApplyCurveOp {
                    result: adjusted_image_id,
                    arg: *current_output_id,
                    control_points: self.curve_control_points.clone(),
                },
            ));
            module.set_output_id(adjusted_image_id);
            *current_output_id = adjusted_image_id;
        }
    }

    fn initial_control_points() -> Vec<(f32, f32)> {
        vec![(0.0, 0.0), (1.0, 1.0)]
    }
}
