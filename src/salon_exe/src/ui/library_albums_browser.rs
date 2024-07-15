use eframe::{
    egui::{self, CollapsingHeader, SelectableLabel, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::session::Session;

use super::{
    ui_set_current_editor_image, utils::get_album_name_text_with_emoji_and_count,
    widgets::ThumbnailCallback, AppUiState,
};

pub fn library_albums_browser(
    ctx: &egui::Context,
    ui: &mut Ui,
    session: &mut Session,
    ui_state: &mut AppUiState,
) {
    ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
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
                    let mut album_to_delete = None;
                    for i in 0..albums.len() {
                        let text = get_album_name_text_with_emoji_and_count(&albums[i]);
                        let response =
                            ui.selectable_label(ui_state.selected_album == Some(i), text);
                        if response.clicked() {
                            ui_state.selected_album = Some(i);
                        } else {
                            response.context_menu(|ui| {
                                // TODO: support deleting albums backed by a directory as well.
                                let can_delete = albums[i].directory.is_none();
                                if ui
                                    .add_enabled(can_delete, egui::Button::new("Delete album"))
                                    .clicked()
                                {
                                    ui.close_menu();
                                    album_to_delete = Some(i);
                                }
                            });
                        }
                    }

                    if let Some(i) = album_to_delete {
                        session.library.delete_album(i);
                        if ui_state.selected_album == Some(i) {
                            ui_state.selected_album = None;
                        }
                    }

                    let mut finished_name_input = false;
                    if let Some(name) = ui_state.new_album_name.as_mut() {
                        let response = ui.add(egui::TextEdit::singleline(name));
                        if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            finished_name_input = true;
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            ui_state.new_album_name = None;
                        }
                    } else {
                        if ui.selectable_label(false, "➕ Create new album").clicked() {
                            ui_state.new_album_name = Some("".to_owned());
                        }
                    }
                    if finished_name_input {
                        let name = ui_state.new_album_name.take().unwrap();
                        if !name.is_empty() {
                            session.library.create_new_album(name);
                        }
                    }
                });
            });
    });
}
