use std::sync::Arc;

use crate::{
    engine::{Engine, ProcessResult},
    image::Image,
    ir::{
        AdjustContrastOp, AdjustExposureOp, AdjustHighlightsAndShadowsOp,
        AdjustTemperatureAndTintOp, AdjustVibranceAndSaturationOp, ApplyCurveOp,
        ComputeBasicStatisticsOp, Id, Module, Op, ComputeHistogramOp, CollectDataForEditorOp, IdTag,
    },
};

pub struct Editor {
    pub current_state: EditorState,
    pub current_input_image: Option<Arc<Image>>,
    pub current_result: Option<ProcessResult>,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            current_state: EditorState::new(),
            current_input_image: None,
            current_result: None,
        }
    }

    pub fn reset_state(&mut self) {
        self.current_state = EditorState::new();
    }

    pub fn execute_edit(&mut self, engine: &mut Engine) {
        if let Some(ref img) = self.current_input_image {
            let module = self.current_state.to_ir_module();
            let result = engine.execute_module(&module, img.clone());
            self.current_result = Some(result);
        }
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

    pub curve_control_points_all: Vec<(f32, f32)>,
    pub curve_control_points_r: Vec<(f32, f32)>,
    pub curve_control_points_g: Vec<(f32, f32)>,
    pub curve_control_points_b: Vec<(f32, f32)>,
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
            curve_control_points_all: EditorState::initial_control_points(),
            curve_control_points_r: EditorState::initial_control_points(),
            curve_control_points_g: EditorState::initial_control_points(),
            curve_control_points_b: EditorState::initial_control_points(),
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
        self.maybe_add_curves(&mut module, &mut current_output_id);

        self.add_collect_data_for_editor(&mut module, &mut current_output_id);

        module
    }

    pub fn add_collect_data_for_editor(&self, module: &mut Module, current_output_id: &mut Id) {

        let histogram_id = module.alloc_id();
        module.push_op(Op::ComputeHistogram(ComputeHistogramOp {
            result: histogram_id,
            arg: *current_output_id,
        }));

        let data_for_editor_id = module.alloc_id();
        module.push_op(Op::CollectDataForEditor(CollectDataForEditorOp {
            result: data_for_editor_id,
            histogram_final: histogram_id,
        }));

        module.set_tagged_id(IdTag::DataForEditor, data_for_editor_id)
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

    fn maybe_add_curves(&self, module: &mut Module, current_output_id: &mut Id) {
        let mut maybe_add_curve = |control: &Vec<(f32, f32)>, r: bool, g: bool, b: bool| {
            if *control != Self::initial_control_points() {
                let adjusted_image_id = module.alloc_id();
                module.push_op(Op::ApplyCurve(ApplyCurveOp {
                    result: adjusted_image_id,
                    arg: *current_output_id,
                    control_points: control.clone(),
                    apply_r: r,
                    apply_g: g,
                    apply_b: b,
                }));
                module.set_output_id(adjusted_image_id);
                *current_output_id = adjusted_image_id;
            }
        };
        maybe_add_curve(&self.curve_control_points_all, true, true, true);
        maybe_add_curve(&self.curve_control_points_r, true, false, false);
        maybe_add_curve(&self.curve_control_points_g, false, true, false);
        maybe_add_curve(&self.curve_control_points_b, false, false, true);
    }

    fn initial_control_points() -> Vec<(f32, f32)> {
        vec![(0.0, 0.0), (1.0, 1.0)]
    }
}
