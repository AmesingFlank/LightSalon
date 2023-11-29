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
            let module = self.current_edit.to_ir_module();
            let result =
                engine.execute_module(&module, img.clone(), &mut self.engine_execution_context);
            self.current_result = Some(result);
        }
    }
}

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

#[derive(Clone, PartialEq)]
pub struct GlobalEdit {
    pub exposure: f32,
    pub contrast: f32,
    pub highlights: f32,
    pub shadows: f32,

    pub curve_control_points_all: Vec<(f32, f32)>,
    pub curve_control_points_r: Vec<(f32, f32)>,
    pub curve_control_points_g: Vec<(f32, f32)>,
    pub curve_control_points_b: Vec<(f32, f32)>,

    pub temperature: f32,
    pub tint: f32,
    pub vibrance: f32,
    pub saturation: f32,

    pub color_mixer_edits: [ColorMixGroup; 8],

    pub dehaze: f32,
    pub vignette: f32,
}

impl GlobalEdit {
    pub fn new() -> Self {
        GlobalEdit {
            exposure: 0.0,
            contrast: 0.0,
            highlights: 0.0,
            shadows: 0.0,

            curve_control_points_all: GlobalEdit::initial_control_points(),
            curve_control_points_r: GlobalEdit::initial_control_points(),
            curve_control_points_g: GlobalEdit::initial_control_points(),
            curve_control_points_b: GlobalEdit::initial_control_points(),

            temperature: 0.0,
            tint: 0.0,
            vibrance: 0.0,
            saturation: 0.0,

            color_mixer_edits: [ColorMixGroup::new(); 8],

            dehaze: 0.0,
            vignette: 0.0,
        }
    }

    pub fn add_edits_to_ir_module(&self, module: &mut Module) {
        let mut current_output_id = module.get_output_id().expect("expecting an output id");

        // do dehaze first, because `PrepareDehaze` is expensive
        self.maybe_add_dehaze(module, &mut current_output_id);

        self.maybe_add_exposure(module, &mut current_output_id);
        self.maybe_add_contrast(module, &mut current_output_id);
        self.maybe_add_highlights_shadows(module, &mut current_output_id);

        self.maybe_add_curves(module, &mut current_output_id);

        self.maybe_add_temperature_tint(module, &mut current_output_id);
        self.maybe_add_vibrance_saturation(module, &mut current_output_id);

        self.maybe_add_color_mix(module, &mut current_output_id);

        self.maybe_add_vignette(module, &mut current_output_id);
    }

    pub fn maybe_add_exposure(&self, module: &mut Module, current_output_id: &mut Id) {
        if self.exposure != 0.0 {
            let exposure_adjusted_image_id = module.alloc_id();
            module.push_op(Op::AdjustExposure(AdjustExposureOp {
                result: exposure_adjusted_image_id,
                arg: *current_output_id,
                exposure: self.exposure,
            }));
            module.set_output_id(exposure_adjusted_image_id);
            *current_output_id = exposure_adjusted_image_id;
        }
    }

    fn maybe_add_contrast(&self, module: &mut Module, current_output_id: &mut Id) {
        if self.contrast != 0.0 {
            let basic_stats_id = module.alloc_id();
            module.push_op(Op::ComputeBasicStatistics(ComputeBasicStatisticsOp {
                result: basic_stats_id,
                arg: *current_output_id,
            }));

            let contrast_adjusted_image_id = module.alloc_id();
            module.push_op(Op::AdjustContrast(AdjustContrastOp {
                result: contrast_adjusted_image_id,
                arg: *current_output_id,
                basic_stats: basic_stats_id,
                contrast: self.contrast,
            }));
            module.set_output_id(contrast_adjusted_image_id);
            *current_output_id = contrast_adjusted_image_id;
        }
    }

    fn maybe_add_highlights_shadows(&self, module: &mut Module, current_output_id: &mut Id) {
        if self.highlights != 0.0 || self.shadows != 0.0 {
            let adjusted_image_id = module.alloc_id();
            module.push_op(Op::AdjustHighlightsAndShadows(
                AdjustHighlightsAndShadowsOp {
                    result: adjusted_image_id,
                    arg: *current_output_id,
                    highlights: self.highlights,
                    shadows: self.shadows,
                },
            ));
            module.set_output_id(adjusted_image_id);
            *current_output_id = adjusted_image_id;
        }
    }

    fn maybe_add_curves(&self, module: &mut Module, current_output_id: &mut Id) {
        let mut maybe_add_curve = |control: &Vec<(f32, f32)>, r: bool, g: bool, b: bool| {
            if *control != Self::initial_control_points() {
                let adjusted_image_id = module.alloc_id();
                module.push_op(Op::ApplyCurve(ApplyCurveOp {
                    result: adjusted_image_id,
                    arg: *current_output_id,
                    control_points: control.clone(),
                    apply_r: r,
                    apply_g: g,
                    apply_b: b,
                }));
                module.set_output_id(adjusted_image_id);
                *current_output_id = adjusted_image_id;
            }
        };
        maybe_add_curve(&self.curve_control_points_all, true, true, true);
        maybe_add_curve(&self.curve_control_points_r, true, false, false);
        maybe_add_curve(&self.curve_control_points_g, false, true, false);
        maybe_add_curve(&self.curve_control_points_b, false, false, true);
    }

    fn maybe_add_temperature_tint(&self, module: &mut Module, current_output_id: &mut Id) {
        if self.temperature != 0.0 || self.tint != 0.0 {
            let temperature_tint_adjusted_image_id = module.alloc_id();
            module.push_op(Op::AdjustTemperatureAndTint(AdjustTemperatureAndTintOp {
                result: temperature_tint_adjusted_image_id,
                arg: *current_output_id,
                temperature: self.temperature,
                tint: self.tint,
            }));
            module.set_output_id(temperature_tint_adjusted_image_id);
            *current_output_id = temperature_tint_adjusted_image_id;
        }
    }

    fn maybe_add_vibrance_saturation(&self, module: &mut Module, current_output_id: &mut Id) {
        if self.vibrance != 0.0 || self.saturation != 0.0 {
            let adjusted_image_id = module.alloc_id();
            module.push_op(Op::AdjustVibranceAndSaturation(
                AdjustVibranceAndSaturationOp {
                    result: adjusted_image_id,
                    arg: *current_output_id,
                    vibrance: self.vibrance,
                    saturation: self.saturation,
                },
            ));
            module.set_output_id(adjusted_image_id);
            *current_output_id = adjusted_image_id;
        }
    }

    fn maybe_add_color_mix(&self, module: &mut Module, current_output_id: &mut Id) {
        if self.color_mixer_edits != [ColorMixGroup::new(); 8] {
            let mut groups = [ColorMixGroup {
                hue: 0.0,
                saturation: 0.0,
                lightness: 0.0,
            }; 8];
            for i in 0..8usize {
                groups[i].hue = self.color_mixer_edits[i].hue;
                groups[i].saturation = self.color_mixer_edits[i].saturation;
                groups[i].lightness = self.color_mixer_edits[i].lightness;
            }
            let adjusted_image_id = module.alloc_id();
            module.push_op(Op::ColorMix(ColorMixOp {
                result: adjusted_image_id,
                arg: *current_output_id,
                groups,
            }));
            module.set_output_id(adjusted_image_id);
            *current_output_id = adjusted_image_id;
        }
    }

    fn maybe_add_vignette(&self, module: &mut Module, current_output_id: &mut Id) {
        if self.vignette != 0.0 {
            let adjusted_image_id = module.alloc_id();
            module.push_op(Op::AdjustVignette(AdjustVignetteOp {
                result: adjusted_image_id,
                arg: *current_output_id,
                vignette: self.vignette,
            }));
            module.set_output_id(adjusted_image_id);
            *current_output_id = adjusted_image_id;
        }
    }

    fn maybe_add_dehaze(&self, module: &mut Module, current_output_id: &mut Id) {
        if self.dehaze != 0.0 {
            let dehazed_id = module.alloc_id();
            module.push_op(Op::PrepareDehaze(PrepareDehazeOp {
                result: dehazed_id,
                arg: *current_output_id,
            }));

            let dehaze_applied_id = module.alloc_id();
            module.push_op(Op::ApplyDehaze(ApplyDehazeOp {
                result: dehaze_applied_id,
                arg: *current_output_id,
                dehazed: dehazed_id,
                amount: self.dehaze,
            }));
            module.set_output_id(dehaze_applied_id);
            *current_output_id = dehaze_applied_id;
        }
    }

    fn initial_control_points() -> Vec<(f32, f32)> {
        vec![(0.0, 0.0), (1.0, 1.0)]
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
