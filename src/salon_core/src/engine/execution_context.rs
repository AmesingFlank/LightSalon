use std::collections::{HashMap, HashSet};

use crate::ir::{Id, Module};

use super::ValueStore;

pub struct ExecutionContext {
    pub value_store: ValueStore,
    last_module: Option<Module>,
    last_input_image_uuid: Option<u32>,
    last_module_result_id_to_op_index: Option<HashMap<Id, usize>>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            value_store: ValueStore::new(),
            last_module: None,
            last_input_image_uuid: None,
            last_module_result_id_to_op_index: None,
        }
    }

    pub fn set_last(&mut self, module: Module, input_image_uuid: u32) {
        self.last_module_result_id_to_op_index = Some(module.get_result_id_to_op_index());
        self.last_module = Some(module);
        self.last_input_image_uuid = Some(input_image_uuid);
    }

    pub fn compute_reusable_ids_set(
        &self,
        new_module: &Module,
        new_image_uuid: u32,
    ) -> HashSet<Id> {
        let mut resuable = HashSet::new();
        if self.last_input_image_uuid != Some(new_image_uuid) {
            return resuable;
        }
        if self.last_module.is_none() {
            return resuable;
        }
        let last_module = self.last_module.as_ref().unwrap();
        let result_id_to_op_index = self.last_module_result_id_to_op_index.as_ref().unwrap();
        for op in new_module.ops().iter() {
            let result_id = op.get_result_id();
            if let Some(last_index) = result_id_to_op_index.get(&result_id) {
                let last_op = &last_module.ops()[*last_index];
                if *op == *last_op {
                    let arg_ids = op.get_arg_ids();
                    let mut all_args_resuable = true;
                    for arg in arg_ids {
                        if !resuable.contains(&arg) {
                            all_args_resuable = false;
                        }
                    }
                    if all_args_resuable {
                        resuable.insert(result_id);
                    }
                }
            }
        }
        resuable
    }
}
