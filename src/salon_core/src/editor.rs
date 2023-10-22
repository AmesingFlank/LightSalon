use crate::ir::{
    AdjustContrastOp, AdjustExposureOp, AdjustSaturationOp, ComputeBasicStatisticsOp, Id, Module,
    Op, AdjustHighlightsOp,
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
    pub saturation_val: f32,
}

impl EditorState {
    pub fn new() -> Self {
        EditorState {
            exposure_val: 0.0,
            contrast_val: 0.0,
            highlights_val: 0.0,
            saturation_val: 0.0,
        }
    }
    pub fn to_ir_module(&self) -> Module {
        let mut module = Module::new_trivial();

        let mut current_output_id = module.get_output_id().expect("expecting an output id");

        self.maybe_add_exposure(&mut module, &mut current_output_id);
        self.maybe_add_contrast(&mut module, &mut current_output_id);
        self.maybe_add_highlights(&mut module, &mut current_output_id);
        self.maybe_add_saturation(&mut module, &mut current_output_id);

        module.add_statistics_ops();

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

    fn maybe_add_highlights(&self, module: &mut Module, current_output_id: &mut Id) {
        if self.highlights_val != 0.0 {
            let highlights_adjusted_image_id = module.alloc_id();
            module.push_op(Op::AdjustHighlights(AdjustHighlightsOp {
                result: highlights_adjusted_image_id,
                arg: *current_output_id,
                highlights: self.highlights_val,
            }));
            module.set_output_id(highlights_adjusted_image_id);
            *current_output_id = highlights_adjusted_image_id;
        }
    }

    fn maybe_add_saturation(&self, module: &mut Module, current_output_id: &mut Id) {
        if self.saturation_val != 0.0 {
            let saturation_adjusted_image_id = module.alloc_id();
            module.push_op(Op::AdjustSaturation(AdjustSaturationOp {
                result: saturation_adjusted_image_id,
                arg: *current_output_id,
                saturation: self.saturation_val,
            }));
            module.set_output_id(saturation_adjusted_image_id);
            *current_output_id = saturation_adjusted_image_id;
        }
    }
}
