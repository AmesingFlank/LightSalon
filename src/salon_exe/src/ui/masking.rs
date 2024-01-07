use std::primitive;

use eframe::{
    egui::{self, CollapsingHeader, Ui},
    egui_wgpu,
};
use salon_core::{
    editor::{Edit, GlobalEdit, MaskedEdit},
    ir::{Mask, MaskPrimitive, MaskTerm, RadialGradientMask},
    session::Session,
};

use super::{utils::get_image_size_in_ui, widgets::MaskIndicatorCallback, AppUiState};

pub fn masking(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState, edit: &mut Edit) {
    CollapsingHeader::new("Masking")
        .default_open(true)
        .show(ui, |ui| {
            ui.group(|ui| {
                egui::Grid::new("my_grid")
                    .num_columns(1)
                    .striped(true)
                    .show(ui, |ui| {
                        for i in 0..edit.masked_edits.len() {
                            if ui.radio(ui_state.selected_mask_index == i, "").clicked() {
                                ui_state.selected_mask_index = i;
                            }
                            if let Some(ref result) = session.editor.current_result {
                                let mask_img = result.masked_edit_results[i].mask.clone();

                                let size = get_image_size_in_ui(ui, &mask_img);
                                let (rect, response) =
                                    ui.allocate_exact_size(size, egui::Sense::click_and_drag());
                                ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                                    rect,
                                    MaskIndicatorCallback {
                                        image: mask_img.clone(),
                                    },
                                ));
                            }
                            ui.label(&edit.masked_edits[i].name);
                            ui.end_row()
                        }
                    });
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
