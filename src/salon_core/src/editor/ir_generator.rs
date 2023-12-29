use super::{Edit, GlobalEdit, MaskedEdit};

use crate::{
    engine::{Engine, ExecutionContext, ProcessResult},
    image::Image,
    ir::{
        AdjustContrastOp, AdjustExposureOp, AdjustHighlightsAndShadowsOp,
        AdjustTemperatureAndTintOp, AdjustVibranceAndSaturationOp, AdjustVignetteOp, ApplyCurveOp,
        ApplyDehazeOp, ApplyMaskedEditsOp, CollectDataForEditorOp, ColorMixGroup, ColorMixOp,
        ComputeBasicStatisticsOp, ComputeHistogramOp, CropOp, Id, IdTag, Module, Op,
        PrepareDehazeOp,
    },
    utils::rectangle::Rectangle,
};

pub fn to_ir_module(edit: &Edit) -> Module {
    let mut module = Module::new_trivial();
    let mut current_output_id = module.get_output_id().expect("expecting an output id");

    maybe_add_crop(edit, &mut module, &mut current_output_id);
    for edit in edit.masked_edits.iter() {
        add_masked_edit(edit, &mut module)
    }

    add_collect_data_for_editor(&mut module, &mut current_output_id);
    module
}

fn maybe_add_crop(edit: &Edit, module: &mut Module, current_output_id: &mut Id) {
    if let Some(ref crop) = edit.crop {
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

fn add_masked_edit(masked_edit: &MaskedEdit, module: &mut Module) {
    let target_id: i32 = module.get_output_id().expect("expecting an output id");

    let mask_id = masked_edit.mask.create_compute_mask_ops(target_id, module);
    add_global_edit(&masked_edit.edit, module);

    let edited_id: i32 = module.get_output_id().expect("expecting an output id");

    let result = module.alloc_id();

    module.push_op(Op::ApplyMaskedEdits(ApplyMaskedEditsOp {
        result,
        mask: mask_id,
        original_target: target_id,
        edited: edited_id,
    }));

    module.set_output_id(result);
}

pub fn add_global_edit(edit: &GlobalEdit, module: &mut Module) {
    let mut current_output_id = module.get_output_id().expect("expecting an output id");

    // do dehaze first, because `PrepareDehaze` is expensive
    maybe_add_dehaze(edit, module, &mut current_output_id);

    maybe_add_exposure(edit, module, &mut current_output_id);
    maybe_add_contrast(edit, module, &mut current_output_id);
    maybe_add_highlights_shadows(edit, module, &mut current_output_id);

    maybe_add_curves(edit, module, &mut current_output_id);

    maybe_add_temperature_tint(edit, module, &mut current_output_id);
    maybe_add_vibrance_saturation(edit, module, &mut current_output_id);

    maybe_add_color_mix(edit, module, &mut current_output_id);

    maybe_add_vignette(edit, module, &mut current_output_id);
}

pub fn maybe_add_exposure(edit: &GlobalEdit, module: &mut Module, current_output_id: &mut Id) {
    if edit.exposure != 0.0 {
        let exposure_adjusted_image_id = module.alloc_id();
        module.push_op(Op::AdjustExposure(AdjustExposureOp {
            result: exposure_adjusted_image_id,
            arg: *current_output_id,
            exposure: edit.exposure,
        }));
        module.set_output_id(exposure_adjusted_image_id);
        *current_output_id = exposure_adjusted_image_id;
    }
}

fn maybe_add_contrast(edit: &GlobalEdit, module: &mut Module, current_output_id: &mut Id) {
    if edit.contrast != 0.0 {
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
            contrast: edit.contrast,
        }));
        module.set_output_id(contrast_adjusted_image_id);
        *current_output_id = contrast_adjusted_image_id;
    }
}

fn maybe_add_highlights_shadows(
    edit: &GlobalEdit,
    module: &mut Module,
    current_output_id: &mut Id,
) {
    if edit.highlights != 0.0 || edit.shadows != 0.0 {
        let adjusted_image_id = module.alloc_id();
        module.push_op(Op::AdjustHighlightsAndShadows(
            AdjustHighlightsAndShadowsOp {
                result: adjusted_image_id,
                arg: *current_output_id,
                highlights: edit.highlights,
                shadows: edit.shadows,
            },
        ));
        module.set_output_id(adjusted_image_id);
        *current_output_id = adjusted_image_id;
    }
}

fn maybe_add_curves(edit: &GlobalEdit, module: &mut Module, current_output_id: &mut Id) {
    let mut maybe_add_curve = |control: &Vec<(f32, f32)>, r: bool, g: bool, b: bool| {
        if *control != GlobalEdit::initial_control_points() {
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
    maybe_add_curve(&edit.curve_control_points_all, true, true, true);
    maybe_add_curve(&edit.curve_control_points_r, true, false, false);
    maybe_add_curve(&edit.curve_control_points_g, false, true, false);
    maybe_add_curve(&edit.curve_control_points_b, false, false, true);
}

fn maybe_add_temperature_tint(edit: &GlobalEdit, module: &mut Module, current_output_id: &mut Id) {
    if edit.temperature != 0.0 || edit.tint != 0.0 {
        let temperature_tint_adjusted_image_id = module.alloc_id();
        module.push_op(Op::AdjustTemperatureAndTint(AdjustTemperatureAndTintOp {
            result: temperature_tint_adjusted_image_id,
            arg: *current_output_id,
            temperature: edit.temperature,
            tint: edit.tint,
        }));
        module.set_output_id(temperature_tint_adjusted_image_id);
        *current_output_id = temperature_tint_adjusted_image_id;
    }
}

fn maybe_add_vibrance_saturation(
    edit: &GlobalEdit,
    module: &mut Module,
    current_output_id: &mut Id,
) {
    if edit.vibrance != 0.0 || edit.saturation != 0.0 {
        let adjusted_image_id = module.alloc_id();
        module.push_op(Op::AdjustVibranceAndSaturation(
            AdjustVibranceAndSaturationOp {
                result: adjusted_image_id,
                arg: *current_output_id,
                vibrance: edit.vibrance,
                saturation: edit.saturation,
            },
        ));
        module.set_output_id(adjusted_image_id);
        *current_output_id = adjusted_image_id;
    }
}

fn maybe_add_color_mix(edit: &GlobalEdit, module: &mut Module, current_output_id: &mut Id) {
    if edit.color_mixer_edits != [ColorMixGroup::new(); 8] {
        let mut groups = [ColorMixGroup {
            hue: 0.0,
            saturation: 0.0,
            lightness: 0.0,
        }; 8];
        for i in 0..8usize {
            groups[i].hue = edit.color_mixer_edits[i].hue;
            groups[i].saturation = edit.color_mixer_edits[i].saturation;
            groups[i].lightness = edit.color_mixer_edits[i].lightness;
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

fn maybe_add_vignette(edit: &GlobalEdit, module: &mut Module, current_output_id: &mut Id) {
    if edit.vignette != 0.0 {
        let adjusted_image_id = module.alloc_id();
        module.push_op(Op::AdjustVignette(AdjustVignetteOp {
            result: adjusted_image_id,
            arg: *current_output_id,
            vignette: edit.vignette,
        }));
        module.set_output_id(adjusted_image_id);
        *current_output_id = adjusted_image_id;
    }
}

fn maybe_add_dehaze(edit: &GlobalEdit, module: &mut Module, current_output_id: &mut Id) {
    if edit.dehaze != 0.0 {
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
            amount: edit.dehaze,
        }));
        module.set_output_id(dehaze_applied_id);
        *current_output_id = dehaze_applied_id;
    }
}
