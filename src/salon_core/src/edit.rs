use crate::op::OpType;

pub struct ActionDesc {
    pub op_type: OpType,
    pub params: serde_json::Value,
}


pub struct EditDesc {
    pub actions: Vec<ActionDesc>,
}