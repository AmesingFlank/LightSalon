use crate::engine::OpType;

pub struct ActionDesc {
    pub op_type: OpType,
    pub params: serde_json::Value,
}
