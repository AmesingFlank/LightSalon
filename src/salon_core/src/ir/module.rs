use std::collections::HashMap;

use super::{id::IdTag, CollectDataForEditorOp, ComputeHistogramOp, Id, InputOp, Op};

#[derive(Clone)]
pub struct Module {
    ops: Vec<Op>,
    next_available_id: Id,
    tagged_ids: HashMap<IdTag, Id>,
}

impl Module {
    pub fn ops(&self) -> &Vec<Op> {
        &self.ops
    }
    pub fn push_op(&mut self, op: Op) {
        self.ops.push(op);
    }
    pub fn get_tagged_id(&self, tag: IdTag) -> Option<Id> {
        self.tagged_ids.get(&tag).copied()
    }
    pub fn set_tagged_id(&mut self, tag: IdTag, id: Id) {
        self.tagged_ids.insert(tag, id);
    }
    pub fn get_output_id(&self) -> Option<Id> {
        self.get_tagged_id(IdTag::Output)
    }
    pub fn set_output_id(&mut self, id: Id) {
        self.set_tagged_id(IdTag::Output, id);
    }
    pub fn new_empty() -> Self {
        Module {
            ops: Vec::new(),
            next_available_id: 0,
            tagged_ids: HashMap::new(),
        }
    }
    pub fn alloc_id(&mut self) -> Id {
        let id = self.next_available_id;
        self.next_available_id += 1;
        id
    }
    pub fn new_trivial() -> Self {
        let mut module = Module::new_empty();
        let input_id = module.alloc_id();
        module.push_op(Op::Input(InputOp { result: input_id }));
        module.set_output_id(input_id);

        module
    }
}
