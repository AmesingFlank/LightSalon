use crate::ir::{
    AdjustContrastOp, AdjustExposureOp, AdjustSaturationOp, ComputeBasicStatisticsOp, Module, Op,
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
    pub saturation_val: f32,
}

impl EditorState {
    pub fn new() -> Self {
        EditorState {
            exposure_val: 0.0,
            contrast_val: 0.0,
            saturation_val: 0.0,
        }
    }
    pub fn to_ir_module(&self) -> Module {
        let mut module = Module::new_trivial();

        let mut current_output_id = module.get_output_id().expect("expecting an output id");

        if self.exposure_val != 0.0 {
            let exposure_adjusted_image_id = module.alloc_id();
            let exposure_op = Op::AdjustExposure(AdjustExposureOp {
                result: exposure_adjusted_image_id,
                arg: current_output_id,
                exposure: self.exposure_val,
            });
            module.push_op(exposure_op);
            module.set_output_id(exposure_adjusted_image_id);
            current_output_id = exposure_adjusted_image_id;
        }

        if self.contrast_val != 0.0 {
            let basic_stats_id = module.alloc_id();
            module.push_op(Op::ComputeBasicStatistics(ComputeBasicStatisticsOp {
                result: basic_stats_id,
                arg: current_output_id,
            }));

            let contrast_adjusted_image_id = module.alloc_id();
            module.push_op(Op::AdjustContrast(AdjustContrastOp {
                result: contrast_adjusted_image_id,
                arg: current_output_id,
                basic_stats: basic_stats_id,
                contrast: self.contrast_val,
            }));
            module.set_output_id(contrast_adjusted_image_id);
            current_output_id = contrast_adjusted_image_id;
        }

        if self.saturation_val != 0.0 {
            let saturation_adjusted_image_id = module.alloc_id();
            let saturation_op = Op::AdjustSaturation(AdjustSaturationOp {
                result: saturation_adjusted_image_id,
                arg: current_output_id,
                saturation: self.saturation_val,
            });
            module.push_op(saturation_op);
            module.set_output_id(saturation_adjusted_image_id);
            current_output_id = saturation_adjusted_image_id;
        }

        module.add_statistics_ops();

        module
    }
}
