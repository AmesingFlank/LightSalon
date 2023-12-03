use super::{ComputeGlobalMaskOp, Id, Module, Op};

#[derive(Clone, PartialEq)]
pub enum Mask {
    Global(GlobalMask),
    //RadialGradient(RadialGradientMask),
}

#[derive(Clone, PartialEq)]
pub struct GlobalMask {}

#[derive(Clone, PartialEq)]
pub struct RadialGradientMask {
    pub center: (f32, f32),
    pub radius_x: f32,
    pub radius_y: f32,
    pub inner_radius_x: f32,
}

impl Mask {
    pub fn create_compute_mask_ops(&self, target: Id, module: &mut Module) -> Id {
        match self {
            Mask::Global(ref m) => m.create_compute_mask_ops(target, module)
        }
    }
}

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