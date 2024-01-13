use crate::ir::{AddMaskOp, InvertMaskOp, SubtractMaskOp};

use super::{ComputeGlobalMaskOp, ComputeRadialGradientMaskOp, Id, Module, Op, ComputeLinearGradientMaskOp};

#[derive(Clone, PartialEq)]
pub enum MaskPrimitive {
    Global(GlobalMask),
    RadialGradient(RadialGradientMask),
    LinearGradient(LinearGradientMask),
}

impl MaskPrimitive {
    pub fn create_compute_mask_ops(&self, target: Id, module: &mut Module) -> Id {
        match self {
            MaskPrimitive::Global(ref m) => m.create_compute_mask_ops(target, module),
            MaskPrimitive::RadialGradient(ref m) => m.create_compute_mask_ops(target, module),
            MaskPrimitive::LinearGradient(ref m) => m.create_compute_mask_ops(target, module),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct MaskTerm {
    pub primitive: MaskPrimitive,
    pub inverted: bool,
    pub subtracted: bool,
}

#[derive(Clone, PartialEq)]
pub struct Mask {
    pub terms: Vec<MaskTerm>,
}

impl Mask {
    pub fn create_compute_mask_ops(&self, target: Id, module: &mut Module) -> (Id, Vec<Id>) {
        assert!(self.terms.len() > 0usize, "mask has no terms!");

        let mut term_ids = Vec::new();
        for term in self.terms.iter() {
            let primitive_id = term.primitive.create_compute_mask_ops(target, module);
            let mut term_id = primitive_id;
            if term.inverted {
                term_id = module.alloc_id();
                module.push_op(Op::InvertMask(InvertMaskOp {
                    result: term_id,
                    mask_0: primitive_id,
                }));
            }
            term_ids.push(term_id);
        }

        assert!(
            !self.terms[0].subtracted,
            "first mask term cannot be subtracted!"
        );

        let mut result_id = term_ids[0];
        for i in 1..term_ids.len() - 1 {
            let new_result = module.alloc_id();
            if self.terms[i].subtracted {
                module.push_op(Op::SubtractMask(SubtractMaskOp {
                    result: new_result,
                    mask_0: result_id,
                    mask_1: term_ids[i],
                }));
            } else {
                module.push_op(Op::AddMask(AddMaskOp {
                    result: new_result,
                    mask_0: result_id,
                    mask_1: term_ids[i],
                }));
            }
            result_id = new_result
        }
        (result_id, term_ids)
    }

    pub fn is_global(&self) -> bool {
        self.terms.len() == 1
            && !self.terms[0].inverted
            && !self.terms[0].subtracted
            && match self.terms[0].primitive {
                MaskPrimitive::Global(_) => true,
                _ => false,
            }
    }
}

#[derive(Clone, PartialEq)]
pub struct GlobalMask {}

impl GlobalMask {
    pub fn create_compute_mask_ops(&self, target: Id, module: &mut Module) -> Id {
        let result = module.alloc_id();
        module.push_op(Op::ComputeGlobalMask(ComputeGlobalMaskOp {
            result,
            mask: self.clone(),
            target,
        }));
        result
    }
}

#[derive(Clone, PartialEq)]
pub struct RadialGradientMask {
    pub center_x: f32,
    pub center_y: f32,
    pub radius_x: f32,
    pub radius_y: f32,
    pub feather: f32,
    pub rotation: f32,
}

impl RadialGradientMask {
    pub fn default(aspect_ratio: f32) -> Self {
        Self {
            center_x: 0.5,
            center_y: 0.5,
            radius_x: 0.1 * aspect_ratio,
            radius_y: 0.1,
            feather: 50.0,
            rotation: 0.0,
        }
    }

    pub fn create_compute_mask_ops(&self, target: Id, module: &mut Module) -> Id {
        let result = module.alloc_id();
        module.push_op(Op::ComputeRadialGradientMask(ComputeRadialGradientMaskOp {
            result,
            mask: self.clone(),
            target,
        }));
        result
    }
}

#[derive(Clone, PartialEq)]
pub struct LinearGradientMask {
    pub begin_x: f32,
    pub begin_y: f32,
    pub saturate_x: f32,
    pub saturate_y: f32,
}

impl LinearGradientMask {
    pub fn default() -> Self {
        Self {
            begin_x: 0.5,
            begin_y: 0.4,
            saturate_x: 0.5,
            saturate_y: 0.6,
        }
    }

    pub fn create_compute_mask_ops(&self, target: Id, module: &mut Module) -> Id {
        let result = module.alloc_id();
        module.push_op(Op::ComputeLinearGradientMask(ComputeLinearGradientMaskOp {
            result,
            mask: self.clone(),
            target,
        }));
        result
    }
}
