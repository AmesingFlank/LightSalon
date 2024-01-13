use std::{primitive, thread::yield_now};

use eframe::{
    egui::{self, CollapsingHeader, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::{
    editor::{Edit, GlobalEdit, MaskedEdit},
    ir::{Mask, MaskPrimitive, MaskTerm, RadialGradientMask},
    session::Session,
};

use super::{
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
            ui.menu_button("Create New Mask", |ui| {
                if ui.button("Radial Gradient").clicked() {
                    let aspect_ratio = session
                        .editor
                        .current_input_image
                        .as_ref()
                        .expect("expecting an input image")
                        .aspect_ratio();
                    add_masked_edit(
                        edit,
                        ui_state,
                        MaskPrimitive::RadialGradient(RadialGradientMask::default(aspect_ratio)),
                    );
                    ui.close_menu();
                }
                if ui.button("Linear Gradient").clicked() {
                    ui.close_menu();
                }
            });
        });
}

pub fn masks_table(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState, edit: &mut Edit) {
    let mut table = TableBuilder::new(ui)
        .column(Column::auto())
        .column(Column::remainder())
        .sense(egui::Sense::click())
        .cell_layout(
            egui::Layout::left_to_right(egui::Align::LEFT).with_cross_align(egui::Align::Center),
        );
    // .cell_layout(
    //     egui::Layout::top_down(egui::Align::Center).with_cross_align(egui::Align::LEFT),
    // );

    table.body(|mut body| {
        for mask_index in 0..edit.masked_edits.len() {
            let image_height = ui_state.last_frame_size.unwrap().1 * 0.03;
            let row_height = image_height * 1.2;
            let is_selected = ui_state.selected_mask_index == mask_index;

            body.row(row_height, |mut row| {
                //row.set_selected(ui_state.selected_mask_index == i);
                row.col(|ui| {
                    if ui.radio(is_selected, "").clicked() {
                        select_mask(ui_state, mask_index);
                    }
                });
                row.col(|ui| {
                    ui.horizontal_centered(|mut ui| {
                        if let Some(ref result) = session.editor.current_result {
                            let mask_img = result.masked_edit_results[mask_index].mask.clone();
                            let aspect_ratio = mask_img.aspect_ratio();
                            let image_width = image_height / aspect_ratio;
                            let size = egui::Vec2 {
                                x: image_width,
                                y: image_height,
                            };
                            let (rect, response) =
                                ui.allocate_exact_size(size, egui::Sense::click_and_drag());
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
                        ui.label(&edit.masked_edits[mask_index].name);
                    });
                });
                if row.response().clicked() {
                    select_mask(ui_state, mask_index);
                }
            });

            if !edit.masked_edits[mask_index].mask.is_global() && is_selected {
                for term_index in 0..edit.masked_edits[mask_index].mask.terms.len() {
                    let term = &mut edit.masked_edits[mask_index].mask.terms[term_index];
                    let row_height = image_height * 1.2;
                    body.row(row_height, |mut row| {
                        if ui_state.selected_mask_term_index == Some(term_index) {
                            row.set_selected(true);
                        }
                        row.col(|ui| {});
                        row.col(|ui| {
                            ui.horizontal_centered(|mut ui| {
                                ui.separator();
                                if let Some(ref result) = session.editor.current_result {
                                    let mask_img = result.masked_edit_results[mask_index]
                                        .mask_terms[term_index]
                                        .clone();
                                    let aspect_ratio = mask_img.aspect_ratio();
                                    let image_width = image_height / aspect_ratio;
                                    let size = egui::Vec2 {
                                        x: image_width,
                                        y: image_height,
                                    };
                                    let (rect, response) =
                                        ui.allocate_exact_size(size, egui::Sense::click_and_drag());
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
                                ui.label(&term_str);
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
                            });
                        }
                    }
                }
            }
        }
    });
}

fn add_masked_edit(edit: &mut Edit, ui_state: &mut AppUiState, primitive: MaskPrimitive) {
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
    }
}
