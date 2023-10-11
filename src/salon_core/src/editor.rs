use crate::ir::{Module, Op, AdjustExposureOp, AdjustSaturationOp};

pub struct Editor {
    pub current_state: EditorState,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            current_state: EditorState::new(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct EditorState {
    pub exposure_val: f32,
    pub saturation_val: f32,
}

impl EditorState {
    pub fn new() -> Self {
        EditorState { 
            exposure_val: 0.0,
            saturation_val: 0.0,
        }
    }
    pub fn to_ir_module(&self) -> Module {
        let mut module = Module::new_trivial();

        let mut current_output_id = module.output_id().expect("expecting an output id");

        let exposure_adjusted_image_id = module.alloc_id();
        let exposure_op = Op::AdjustExposure(AdjustExposureOp {
            result: exposure_adjusted_image_id,
            arg: current_output_id,
            exposure: self.exposure_val,
        });
        module.push_op(exposure_op);
        module.set_output_id(exposure_adjusted_image_id);

        current_output_id = exposure_adjusted_image_id;

        let saturation_adjusted_image_id = module.alloc_id();
        let saturation_op = Op::AdjustSaturation(AdjustSaturationOp {
            result: saturation_adjusted_image_id,
            arg: current_output_id,
            saturation: self.saturation_val,
        });
        module.push_op(saturation_op);
        module.set_output_id(saturation_adjusted_image_id);

        module
    }
}
