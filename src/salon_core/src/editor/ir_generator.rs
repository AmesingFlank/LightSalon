use super::{Edit, GlobalEdit, MaskedEdit};

use crate::{ir::{
    AdjustContrastOp, AdjustExposureOp, AdjustHighlightsAndShadowsOp, AdjustTemperatureAndTintOp,
    AdjustVibranceAndSaturationOp, AdjustVignetteOp, ApplyCurveOp, ApplyDehazeOp,
    ApplyMaskedEditsOp, ColorMixGroup, ColorMixOp, ComputeBasicStatisticsOp, ComputeHistogramOp,
    Id, InputOp, Module, Op, PrepareDehazeOp, ResizeOp, RotateAndCropOp,
}, utils::rectangle::Rectangle};

pub struct IdStore {
    pub final_image: Id,
    pub geometry_only: Id,
    pub final_histogram: Id,
    pub masked_edit_id_stores: Vec<MaskedEditIdStore>,
}

pub struct MaskedEditIdStore {
    pub mask_id: Id,
    pub term_ids: Vec<Id>,
    pub result_image_id: Id,
}

pub fn to_ir_module(edit: &Edit) -> (Module, IdStore) {
    let mut module = Module::new_empty();

    let input_id = module.alloc_id();
    module.push_op(Op::Input(InputOp { result: input_id }));

    let mut current_output_id = input_id;

    maybe_add_resize(edit, &mut module, &mut current_output_id);
    maybe_add_rotate_and_crop(edit, &mut module, &mut current_output_id);

    let geometry_only = current_output_id;

    let mut masked_edit_id_stores = Vec::new();
    for edit in edit.masked_edits.iter() {
        let masked_id_store = add_masked_edit(edit, &mut module, current_output_id);
        current_output_id = masked_id_store.result_image_id;
        masked_edit_id_stores.push(masked_id_store);
    }

    let final_histogram_id = add_final_histogram(&mut module, &current_output_id);
    let id_store = IdStore {
        final_image: current_output_id,
        geometry_only,
        final_histogram: final_histogram_id,
        masked_edit_id_stores,
    };
    // println!("{:#?}", module.ops());
    (module, id_store)
}

fn maybe_add_resize(edit: &Edit, module: &mut Module, current_output_id: &mut Id) {
    if let Some(ref factor) = edit.resize_factor {
        if *factor != 1.0 {
            let resized_image_id = module.alloc_id();
            module.push_op(Op::Resize(ResizeOp {
                result: resized_image_id,
                arg: *current_output_id,
                factor: *factor,
            }));
            *current_output_id = resized_image_id;
        }
    }
}

fn maybe_add_rotate_and_crop(edit: &Edit, module: &mut Module, current_output_id: &mut Id) {
    if edit.resize_factor.is_some() || edit.crop_rect.is_some() {
        let cropped_image_id = module.alloc_id();
        module.push_op(Op::RotateAndCrop(RotateAndCropOp {
            result: cropped_image_id,
            arg: *current_output_id,
            rotation_degrees: edit.rotation_degrees.clone().unwrap_or(0.0),
            crop_rect: edit.crop_rect.clone().unwrap_or(Rectangle::regular()),
        }));
        *current_output_id = cropped_image_id;
    }
}

fn add_final_histogram(module: &mut Module, current_output_id: &Id) -> Id {
    let histogram_id = module.alloc_id();
    module.push_op(Op::ComputeHistogram(ComputeHistogramOp {
        result: histogram_id,
        arg: *current_output_id,
    }));

    histogram_id
}

fn add_masked_edit(
    masked_edit: &MaskedEdit,
    module: &mut Module,
    target_id: Id,
) -> MaskedEditIdStore {
    let (mask_id, term_ids) = masked_edit.mask.create_compute_mask_ops(target_id, module);
    let edited_id = add_global_edit(&masked_edit.edit, module, target_id);

    let result_image_id = module.alloc_id();

    module.push_op(Op::ApplyMaskedEdits(ApplyMaskedEditsOp {
        result: result_image_id,
        mask: mask_id,
        original_target: target_id,
        edited: edited_id,
    }));

    MaskedEditIdStore {
        mask_id,
        term_ids,
        result_image_id,
    }
}

pub fn add_global_edit(edit: &GlobalEdit, module: &mut Module, target_id: Id) -> Id {
    // do dehaze first, because `PrepareDehaze` is expensive
    let mut current_output_id = target_id;
    maybe_add_dehaze(edit, module, &mut current_output_id);

    maybe_add_exposure(edit, module, &mut current_output_id);
    maybe_add_contrast(edit, module, &mut current_output_id);
    maybe_add_highlights_shadows(edit, module, &mut current_output_id);

    maybe_add_curves(edit, module, &mut current_output_id);

    maybe_add_temperature_tint(edit, module, &mut current_output_id);
    maybe_add_vibrance_saturation(edit, module, &mut current_output_id);

    maybe_add_color_mix(edit, module, &mut current_output_id);

    maybe_add_vignette(edit, module, &mut current_output_id);
    current_output_id
}

pub fn maybe_add_exposure(edit: &GlobalEdit, module: &mut Module, current_output_id: &mut Id) {
    if edit.exposure != 0.0 {
        let exposure_adjusted_image_id = module.alloc_id();
        module.push_op(Op::AdjustExposure(AdjustExposureOp {
            result: exposure_adjusted_image_id,
            arg: *current_output_id,
            exposure: edit.exposure,
        }));
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
        *current_output_id = dehaze_applied_id;
    }
}
