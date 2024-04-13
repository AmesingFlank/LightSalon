use std::{primitive, thread::yield_now};

use eframe::{
    egui::{self, CollapsingHeader, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::{
    editor::{Edit, GlobalEdit, MaskedEdit},
    ir::{GlobalMask, LinearGradientMask, Mask, MaskPrimitive, MaskTerm, RadialGradientMask},
    session::Session,
};

use super::{
    utils::get_max_image_size,
    widgets::{EditorSlider, MaskIndicatorCallback},
    AppUiState,
};

pub fn masking(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState, edit: &mut Edit) {
    CollapsingHeader::new("Masking")
        .default_open(true)
        .show(ui, |ui| {
            ui.group(|ui| {
                masks_table(ui, session, ui_state, edit);
            });
            new_mask_menu_button(ui, edit, session, ui_state);
        });
}

pub fn masks_table(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState, edit: &mut Edit) {
    let image_aspect_ratio = session
        .editor
        .current_edit_context_ref()
        .unwrap()
        .input_image()
        .aspect_ratio();

    let mut mask_aspect_ratio = image_aspect_ratio;
    if let Some(crop_rect) = edit.crop_rect {
        mask_aspect_ratio *= crop_rect.size.y / crop_rect.size.x
    }

    let mask_max_width = ui.available_width() * 0.2;
    let mask_max_height = mask_max_width * 0.5;
    let mask_indicator_size =
        get_max_image_size(mask_aspect_ratio, mask_max_width, mask_max_height);
    let row_height = mask_max_height * 1.1;

    let column_widths = vec![
        ui.available_width() * 0.1,
        ui.available_width() * 0.75,
        ui.available_width() * 0.1,
    ];

    let table = TableBuilder::new(ui)
        .column(Column::auto().range(egui::Rangef::point(column_widths[0])))
        .column(Column::auto().range(egui::Rangef::point(column_widths[1])))
        .column(Column::auto().range(egui::Rangef::point(column_widths[2])))
        .sense(egui::Sense::click())
        .cell_layout(
            egui::Layout::left_to_right(egui::Align::LEFT).with_cross_align(egui::Align::Center),
        );

    let mut mask_to_delete: Option<usize> = None;
    let mut mask_term_to_delete: Option<(usize, usize)> = None;
    let mut mask_to_duplicate: Option<usize> = None;
    let mut mask_term_to_duplicate: Option<(usize, usize)> = None;

    table.body(|mut body| {
        for mask_index in 0..edit.masked_edits.len() {
            let is_selected = ui_state.selected_mask_index == mask_index;
            // row for the entire mask
            body.row(row_height, |mut row| {
                //row.set_selected(ui_state.selected_mask_index == i);
                row.col(|ui| {
                    if ui.radio(is_selected, "").clicked() {
                        select_mask(ui_state, mask_index);
                    }
                });
                row.col(|ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                        ui.horizontal_centered(|mut ui| {
                            if let Some(ref result) = session
                                .editor
                                .current_edit_context_ref()
                                .unwrap()
                                .current_result
                            {
                                let mask_img = result.masked_edit_results[mask_index].mask.clone();
                                let (rect, response) = ui.allocate_exact_size(
                                    mask_indicator_size,
                                    egui::Sense::click_and_drag(),
                                );
                                ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                                    rect,
                                    MaskIndicatorCallback {
                                        image: mask_img.clone(),
                                    },
                                ));

                                if response.clicked() {
                                    select_mask(ui_state, mask_index);
                                }
                            }
                            if ui.label(&edit.masked_edits[mask_index].name).clicked() {
                                select_mask(ui_state, mask_index);
                            }
                        });
                    });
                });
                row.col(|ui| {
                    ui.menu_button(menu_dots(), |ui| {
                        if ui.button("Delete").clicked() {
                            mask_to_delete = Some(mask_index);
                            ui.close_menu();
                        }
                        if ui.button("Duplicate").clicked() {
                            mask_to_duplicate = Some(mask_index);
                            ui.close_menu();
                        }
                    });
                });
                if row.response().clicked() {
                    select_mask(ui_state, mask_index);
                }
            });

            // rows for each term of the mask.
            if !edit.masked_edits[mask_index].mask.is_singe_global() && is_selected {
                for term_index in 0..edit.masked_edits[mask_index].mask.terms.len() {
                    let term = &mut edit.masked_edits[mask_index].mask.terms[term_index];
                    body.row(row_height, |mut row| {
                        if ui_state.selected_mask_term_index == Some(term_index) {
                            row.set_selected(true);
                        }
                        row.col(|ui| {});
                        row.col(|ui| {
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                                ui.horizontal_centered(|mut ui| {
                                    ui.separator();
                                    if let Some(ref result) = session
                                        .editor
                                        .current_edit_context_ref()
                                        .unwrap()
                                        .current_result
                                    {
                                        let mask_img = result.masked_edit_results[mask_index]
                                            .mask_terms[term_index]
                                            .clone();
                                        let (rect, response) = ui.allocate_exact_size(
                                            mask_indicator_size,
                                            egui::Sense::click_and_drag(),
                                        );
                                        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                                            rect,
                                            MaskIndicatorCallback {
                                                image: mask_img.clone(),
                                            },
                                        ));
                                        if response.clicked() {
                                            maybe_select_term(ui_state, term_index);
                                        }
                                    }
                                    let mut term_str =
                                        mask_primtive_type_str(&term.primitive).to_string();
                                    if term.subtracted {
                                        term_str += " (Subtracted)"
                                    }
                                    if term.inverted {
                                        term_str += " (Inverted)"
                                    }
                                    if ui.label(&term_str).clicked() {
                                        maybe_select_term(ui_state, term_index);
                                    }
                                });
                            });
                        });
                        row.col(|ui| {
                            ui.menu_button(menu_dots(), |ui| {
                                if ui.button("Invert").clicked() {
                                    term.inverted = !term.inverted;
                                    ui.close_menu();
                                }
                                if ui.button("Delete").clicked() {
                                    mask_term_to_delete = Some((mask_index, term_index));
                                    ui.close_menu();
                                }
                                if ui.button("Duplicate").clicked() {
                                    mask_term_to_duplicate = Some((mask_index, term_index));
                                    ui.close_menu();
                                }
                            });
                        });
                        if row.response().clicked() {
                            maybe_select_term(ui_state, term_index);
                        }
                    });
                    if ui_state.selected_mask_term_index == Some(term_index) {
                        if let MaskPrimitive::RadialGradient(ref mut radial) = &mut term.primitive {
                            body.row(row_height, |mut row| {
                                row.set_selected(true);
                                row.col(|ui| {});
                                row.col(|ui| {
                                    ui.horizontal_centered(|mut ui| {
                                        ui.separator();
                                        ui.add(
                                            EditorSlider::new(&mut radial.feather, 0.0..=100.0)
                                                .text("Feather"),
                                        );
                                    });
                                });
                                row.col(|ui| {});
                            });
                        }
                    }
                }
            }

            // row for adding/subtracting terms
            if is_selected {
                body.row(row_height, |mut row| {
                    row.col(|ui| {});
                    row.col(|ui| {
                        ui.horizontal_centered(|mut ui| {
                            if !edit.masked_edits[mask_index].mask.is_singe_global() {
                                new_mask_term_menu_button(
                                    ui,
                                    ui_state,
                                    &mut edit.masked_edits[mask_index].mask,
                                    mask_aspect_ratio,
                                    false,
                                );
                            }
                            new_mask_term_menu_button(
                                ui,
                                ui_state,
                                &mut edit.masked_edits[mask_index].mask,
                                mask_aspect_ratio,
                                true,
                            );
                        });
                    });
                    row.col(|ui| {});
                });
            }
        }
    });

    if let Some((m, t)) = mask_term_to_delete {
        edit.masked_edits[m].mask.terms.remove(t);
        if edit.masked_edits[m].mask.terms.is_empty() {
            edit.masked_edits.remove(m);
            ui_state.selected_mask_index = 0;
        }
        ui_state.selected_mask_term_index = None
    } else if let Some(m) = mask_to_delete {
        edit.masked_edits.remove(m);
        ui_state.selected_mask_index = 0;
        ui_state.selected_mask_term_index = None
    } else if let Some((m, t)) = mask_term_to_duplicate {
        let term = edit.masked_edits[m].mask.terms[t].clone();
        edit.masked_edits[m].mask.terms.insert(t + 1, term);
        ui_state.selected_mask_term_index = Some(t + 1);
    } else if let Some(m) = mask_to_duplicate {
        let mask = edit.masked_edits[m].mask.clone();
        edit.masked_edits.insert(
            m + 1,
            MaskedEdit {
                mask,
                edit: GlobalEdit::new(),
                name: edit.masked_edits[m].name.clone() + " Copy",
            },
        );
        ui_state.selected_mask_index = m + 1;
    }
}

fn new_mask_menu_button(
    ui: &mut Ui,
    edit: &mut Edit,
    session: &mut Session,
    ui_state: &mut AppUiState,
) {
    ui.menu_button("Create New Mask", |ui| {
        if ui.button("Radial Gradient").clicked() {
            let mut aspect_ratio = session
                .editor
                .current_edit_context_ref()
                .expect("expecting an input image")
                .input_image()
                .aspect_ratio();
            if let Some(crop_rect) = edit.crop_rect {
                aspect_ratio *= crop_rect.size.y / crop_rect.size.x
            }
            add_single_primitive_masked_edit(
                edit,
                ui_state,
                MaskPrimitive::RadialGradient(RadialGradientMask::default(aspect_ratio)),
            );
            ui.close_menu();
        }
        if ui.button("Linear Gradient").clicked() {
            add_single_primitive_masked_edit(
                edit,
                ui_state,
                MaskPrimitive::LinearGradient(LinearGradientMask::default()),
            );
            ui.close_menu();
        }
        if ui.button("Global").clicked() {
            add_single_primitive_masked_edit(
                edit,
                ui_state,
                MaskPrimitive::Global(GlobalMask::default()),
            );
            ui.close_menu();
        }
    });
}

fn new_mask_term_menu_button(
    ui: &mut Ui,
    ui_state: &mut AppUiState,
    mask: &mut Mask,
    aspect_ratio: f32,
    subtracted: bool,
) {
    let button_name = if subtracted { "Subtract" } else { "Add" };
    ui.menu_button(button_name, |ui| {
        if ui.button("Radial Gradient").clicked() {
            mask.terms.push(MaskTerm {
                primitive: MaskPrimitive::RadialGradient(RadialGradientMask::default(aspect_ratio)),
                inverted: false,
                subtracted,
            });
            ui_state.selected_mask_term_index = Some(mask.terms.len() - 1);
            ui.close_menu();
        }
        if ui.button("Linear Gradient").clicked() {
            mask.terms.push(MaskTerm {
                primitive: MaskPrimitive::LinearGradient(LinearGradientMask::default()),
                inverted: false,
                subtracted,
            });
            ui_state.selected_mask_term_index = Some(mask.terms.len() - 1);
            ui.close_menu();
        }
    });
}

fn add_single_primitive_masked_edit(
    edit: &mut Edit,
    ui_state: &mut AppUiState,
    primitive: MaskPrimitive,
) {
    let added_index = edit.masked_edits.len();
    let name = "Mask ".to_string() + added_index.to_string().as_str();
    edit.masked_edits.push(MaskedEdit {
        mask: Mask {
            terms: vec![MaskTerm {
                primitive,
                inverted: false,
                subtracted: false,
            }],
        },
        edit: GlobalEdit::new(),
        name,
    });
    ui_state.selected_mask_index = added_index;
    ui_state.selected_mask_term_index = Some(0);
}

fn mask_primtive_type_str(primitive: &MaskPrimitive) -> &str {
    match primitive {
        MaskPrimitive::Global(_) => "Global",
        MaskPrimitive::RadialGradient(_) => "Radial",
        MaskPrimitive::LinearGradient(_) => "Linear",
    }
}

fn select_mask(ui_state: &mut AppUiState, mask_index: usize) {
    ui_state.selected_mask_index = mask_index;
    ui_state.selected_mask_term_index = None;
}

fn maybe_select_term(ui_state: &mut AppUiState, term_index: usize) {
    if ui_state.selected_mask_term_index == Some(term_index) {
        ui_state.selected_mask_term_index = None;
    } else {
        ui_state.selected_mask_term_index = Some(term_index);

        // this enters the term-edit mode, which requires mask primitive control points to be drawn
        // at the moment, the mask primitive drawing and editting logic in main_image assumes the image is not zoomed
        ui_state.main_image_zoom = None;
    }
}

fn menu_dots() -> String {
    "•••".to_owned()
}
