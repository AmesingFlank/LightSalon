use crate::ir::{CollectDataForEditorOp, ComputeHistogramOp, CropOp, Id, IdTag, Module, Op};

use crate::utils::rectangle::Rectangle;

use super::GlobalEdit;

#[derive(Clone, PartialEq)]
pub struct Edit {
    pub crop: Option<Rectangle>,
    pub global: GlobalEdit,
    // TODO: masks
}

impl Edit {
    pub fn new() -> Self {
        Self {
            crop: None,
            global: GlobalEdit::new(),
        }
    }

    pub fn to_ir_module(&self) -> Module {
        let mut module = Module::new_trivial();
        let mut current_output_id = module.get_output_id().expect("expecting an output id");

        self.maybe_add_crop(&mut module, &mut current_output_id);
        self.global.add_edits_to_ir_module(&mut module);

        add_collect_data_for_editor(&mut module, &mut current_output_id);
        module
    }

    pub fn maybe_add_crop(&self, module: &mut Module, current_output_id: &mut Id) {
        if let Some(ref crop) = self.crop {
            let cropped_image_id = module.alloc_id();
            module.push_op(Op::Crop(CropOp {
                result: cropped_image_id,
                arg: *current_output_id,
                rect: crop.clone(),
            }));
            module.set_output_id(cropped_image_id);
            *current_output_id = cropped_image_id;
        }
    }
}

fn add_collect_data_for_editor(module: &mut Module, current_output_id: &mut Id) {
    let histogram_id = module.alloc_id();
    module.push_op(Op::ComputeHistogram(ComputeHistogramOp {
        result: histogram_id,
        arg: *current_output_id,
    }));

    let data_for_editor_id = module.alloc_id();
    module.push_op(Op::CollectDataForEditor(CollectDataForEditorOp {
        result: data_for_editor_id,
        histogram_final: histogram_id,
    }));

    module.set_tagged_id(IdTag::DataForEditor, data_for_editor_id)
}
