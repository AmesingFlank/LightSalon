use std::primitive;

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

use super::{widgets::MaskIndicatorCallback, AppUiState};

pub fn masking(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState, edit: &mut Edit) {
    CollapsingHeader::new("Masking")
        .default_open(true)
        .show(ui, |ui| {
            ui.group(|ui| {
                masks_table(ui, session, ui_state, edit);
            });
            ui.menu_button("Create New Mask", |ui| {
                if ui.button("Radial Gradient").clicked() {
                    add_masked_edit(
                        edit,
                        MaskPrimitive::RadialGradient(RadialGradientMask::default()),
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
                        ui_state.selected_mask_index = mask_index;
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
                        }
                        ui.label(&edit.masked_edits[mask_index].name);
                    });
                });
                if row.response().clicked() {
                    ui_state.selected_mask_index = mask_index;
                }
            });

            if !edit.masked_edits[mask_index].mask.is_global() && is_selected {
                for term_index in 0..edit.masked_edits[mask_index].mask.terms.len() {
                    let row_height = image_height * 1.2;
                    body.row(row_height, |mut row| {
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
                                }
                            });
                        });
                    });
                }
            }
        }
    });
}

fn add_masked_edit(edit: &mut Edit, primitive: MaskPrimitive) {
    let added_index = edit.masked_edits.len();
    let name = "Custom Mask ".to_string() + added_index.to_string().as_str();
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
}
