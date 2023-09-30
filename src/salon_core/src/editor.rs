use crate::ir::{Module, Op, ExposureAdjust};

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

#[derive(Clone)]
pub struct EditorState {
    pub exposure_val: f32,
}

impl EditorState {
    pub fn new() -> Self {
        EditorState { exposure_val: 0.0 }
    }
    pub fn to_ir_module(&self) -> Module {
        let mut module = Module::new_trivial();
        let current_output_id = module.output_id().expect("expecting an output id");
        let exposure_adjusted_image_id = module.alloc_id();
        let op = Op::ExposureAdjust(ExposureAdjust {
            result: exposure_adjusted_image_id,
            arg: current_output_id,
            exposure: self.exposure_val,
        });
        module.push_op(op);
        module.set_output_id(exposure_adjusted_image_id);
        module
    }
}
