use std::collections::HashMap;

use super::{CollectDataForEditorOp, ComputeHistogramOp, Id, InputOp, Op};

#[derive(Clone)]
pub struct Module {
    ops: Vec<Op>,
    next_available_id: Id,
}

impl Module {
    pub fn ops(&self) -> &Vec<Op> {
        &self.ops
    }
    pub fn push_op(&mut self, op: Op) {
        self.ops.push(op);
    }
    pub fn new_empty() -> Self {
        Module {
            ops: Vec::new(),
            next_available_id: 0,
        }
    }
    pub fn alloc_id(&mut self) -> Id {
        let id = self.next_available_id;
        self.next_available_id += 1;
        id
    }

    pub fn get_result_id_to_op_index(&self) -> HashMap<Id, usize> {
        let mut mapping = HashMap::new();
        for i in 0..self.ops().len() {
            let result_id = self.ops()[i].get_result_id();
            mapping.insert(result_id, i);
        }
        mapping
    }
}
