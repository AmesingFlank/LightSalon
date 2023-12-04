use eframe::{egui::{self, CollapsingHeader, Ui}, egui_wgpu};
use salon_core::session::Session;

use super::AppUiState;

pub fn masking(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    CollapsingHeader::new("Masking")
        .default_open(true)
        .show(ui, |ui| {
            ui.group(|ui| {
                egui::Grid::new("my_grid").num_columns(1).show(ui, |ui| {
                    for mask in session.editor.current_edit.masked_edits.iter() {
                        // let rect = ui.available_height();
                        // ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                        //     rect,
                        //     ThumbnailCallback { image: image },
                        // ));
                        ui.label("Mask");
                        ui.end_row()
                    }
                });
            })
        });
}
