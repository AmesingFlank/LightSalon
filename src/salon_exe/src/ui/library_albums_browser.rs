use eframe::{
    egui::{self, CollapsingHeader, SelectableLabel, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::session::Session;

use super::{ui_set_current_editor_image, widgets::ThumbnailCallback, AppUiState};

pub fn library_albums_browser(
    ctx: &egui::Context,
    ui: &mut Ui,
    session: &mut Session,
    ui_state: &mut AppUiState,
) {
    ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
        ui.separator();
        if ui.selectable_label(false, "➕ Import Image").clicked() {
            ui_state.import_image_dialog.open_pick_images();
        }

        #[cfg(not(target_arch = "wasm32"))]
        if ui
            .selectable_label(false, "➕ Import Folder as Album")
            .clicked()
        {
            ui_state.import_image_dialog.open_pick_folder();
        }
        
        ui.separator();
        let all_photos_text = "🖼 All Photos".to_owned()
            + " ("
            + session.library.num_images_total().to_string().as_str()
            + ")";
        if ui
            .selectable_label(ui_state.selected_album.is_none(), all_photos_text)
            .clicked()
        {
            ui_state.selected_album = None;
        }
        ui.separator();

        CollapsingHeader::new("Albums")
            .default_open(true)
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    let albums = session.library.albums();
                    for i in 0..albums.len() {
                        let text = "📷 ".to_owned()
                            + albums[i].name.as_str()
                            + " ("
                            + albums[i].all_images_ordered.len().to_string().as_str()
                            + ")";
                        if ui
                            .selectable_label(ui_state.selected_album == Some(i), text)
                            .clicked()
                        {
                            ui_state.selected_album = Some(i);
                        }
                    }
                });
            });
    });
}
