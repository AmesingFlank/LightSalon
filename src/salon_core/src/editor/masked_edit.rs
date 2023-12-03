use crate::ir::{Mask, Module, Op, ApplyMaskedEditsOp};

use super::GlobalEdit;

#[derive(Clone, PartialEq)]
pub struct MaskedEdit {
    pub mask: Mask,
    pub edit: GlobalEdit,
}

impl MaskedEdit {
    pub fn new(mask: Mask, edit: GlobalEdit) -> Self {
        Self { mask, edit }
    }

    pub fn add_edits_to_ir_module(&self, module: &mut Module) {
        let target_id: i32 = module.get_output_id().expect("expecting an output id");

        let mask_id = self.mask.create_compute_mask_ops(target_id, module);
        self.edit.add_edits_to_ir_module(module);

        let edited_id: i32 = module.get_output_id().expect("expecting an output id");

        let result = module.alloc_id();

        module.push_op(Op::ApplyMaskedEdits(ApplyMaskedEditsOp {
            result,
            mask:mask_id,
            original_target:target_id,
            edited: edited_id,
        }));

        module.set_output_id(result);
    }
}
