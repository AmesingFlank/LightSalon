use super::{Id, Input, Op};

pub struct Module {
    ops: Vec<Op>,
    output_id: Option<Id>,
    next_id: Id,
}

impl Module {
    pub fn ops(&self) -> &Vec<Op> {
        &self.ops
    }
    pub fn push_op(&mut self, op: Op) {
        self.ops.push(op);
    }
    pub fn output_id(&self) -> Option<Id> {
        self.output_id.clone()
    }
    pub fn set_output_id(&mut self, id: Id) {
        self.output_id = Some(id)
    }
    pub fn new_empty() -> Self {
        Module {
            ops: Vec::new(),
            output_id: None,
            next_id: 0,
        }
    }
    pub fn alloc_id(&mut self) -> Id {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    pub fn new_trivial() -> Self {
        let mut module = Module::new_empty();
        let input_id = module.alloc_id();
        module.push_op(Op::Input(Input { result: input_id }));
        module.set_output_id(input_id);
        module
    }
    pub fn clone(&self) -> Self {
        let mut ops: Vec<Op> = Vec::new();
        for o in self.ops() {
            ops.push(o.clone());
        }
        Module {
            ops,
            output_id: self.output_id.clone(),
            next_id: self.next_id.clone(),
        }
    }
}
