use eframe::{
    egui::{self, CollapsingHeader, Ui},
    egui_wgpu,
};
use salon_core::session::Session;

use super::{utils::get_image_size_in_ui, widgets::MaskIndicatorCallback, AppUiState};

pub fn masking(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    CollapsingHeader::new("Masking")
        .default_open(true)
        .show(ui, |ui| {
            ui.group(|ui| {
                egui::Grid::new("my_grid").num_columns(1).show(ui, |ui| {
                    for i in 0..session.editor.current_edit.masked_edits.len() {
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
                        ui.label("Mask");
                        ui.end_row()
                    }
                });
            })
        });
}
