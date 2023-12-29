use std::sync::Arc;

use crate::{
    engine::{Engine, ExecutionContext, ProcessResult},
    image::Image,
    ir::{
        AdjustContrastOp, AdjustExposureOp, AdjustHighlightsAndShadowsOp,
        AdjustTemperatureAndTintOp, AdjustVibranceAndSaturationOp, AdjustVignetteOp, ApplyCurveOp,
        ApplyDehazeOp, CollectDataForEditorOp, ColorMixGroup, ColorMixOp, ComputeBasicStatisticsOp,
        ComputeHistogramOp, CropOp, Id, IdTag, Module, Op, PrepareDehazeOp,
    },
    utils::rectangle::Rectangle,
};

use super::{Edit, ir_generator::to_ir_module};

pub struct Editor {
    pub current_edit: Edit,
    pub current_input_image: Option<Arc<Image>>,
    pub current_result: Option<ProcessResult>,
    engine_execution_context: ExecutionContext,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            current_edit: Edit::new(),
            current_input_image: None,
            current_result: None,
            engine_execution_context: ExecutionContext::new(),
        }
    }

    pub fn reset_state(&mut self) {
        self.current_edit = Edit::new();
    }

    pub fn execute_edit(&mut self, engine: &mut Engine) {
        if let Some(ref img) = self.current_input_image {
            let module = to_ir_module(&self.current_edit);
            let result =
                engine.execute_module(&module, img.clone(), &mut self.engine_execution_context);
            self.current_result = Some(result);
        }
    }
}
