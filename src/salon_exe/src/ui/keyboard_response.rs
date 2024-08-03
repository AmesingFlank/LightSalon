use eframe::{
    egui::{self, Modifiers, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::session::Session;

use super::{
    export_panel::exit_export_panel, redo_action, ui_set_current_editor_image, undo_action, utils::legalize_ui_state, AppPage, AppUiState, EditorPanel
};

pub fn keyboard_response(ctx: &egui::Context, session: &mut Session, ui_state: &mut AppUiState) {
    ctx.input(|i| {
        if i.key_pressed(egui::Key::Escape) {
            match ui_state.app_page {
                AppPage::Export => {
                    exit_export_panel(ui_state);
                }
                AppPage::Editor => {
                    ui_state.app_page = AppPage::Library;
                }
                AppPage::Library => {}
            }
        }
        if ui_state.app_page == AppPage::Editor {
            if i.key_pressed(egui::Key::Enter) {
                if ui_state.editor_panel == EditorPanel::CropAndRotate {
                    ui_state.editor_panel = EditorPanel::LightAndColor;
                    if session.editor.commit_transient_edit(true) {
                        session.update_thumbnail_for_current_image();
                    }
                }
            }
            let control_or_comand = i.modifiers.command || i.modifiers.ctrl || i.modifiers.mac_cmd;
            if i.key_pressed(egui::Key::Z) && control_or_comand {
                if i.modifiers.shift {
                    redo_action(session, ui_state)
                } else {
                    undo_action(session, ui_state);
                }
            }

            if let Some(current_row) = ui_state.library_side_panel_current_row {
                let num_images = if let Some(album_index) = ui_state.selected_album {
                    session.library.num_images_in_album(album_index)
                } else {
                    session.library.num_images_total()
                };
                if i.key_pressed(egui::Key::ArrowUp) && current_row > 0 {
                    ui_state.library_side_panel_requested_row = Some(current_row - 1);
                }
                if i.key_pressed(egui::Key::ArrowDown) && current_row < num_images - 1 {
                    ui_state.library_side_panel_requested_row = Some(current_row + 1);
                }
                if let Some(ref requested_row) = ui_state.library_side_panel_requested_row {
                    let requested_image_identifier =
                        if let Some(album_index) = ui_state.selected_album {
                            session
                                .library
                                .get_identifier_at_index_for_album(*requested_row, album_index)
                                .clone()
                        } else {
                            session
                                .library
                                .get_identifier_at_index(*requested_row)
                                .clone()
                        };
                    if session.editor.current_image_identifier().is_none()
                        || session.editor.current_image_identifier().unwrap()
                            != requested_image_identifier
                    {
                        ui_set_current_editor_image(
                            ctx,
                            session,
                            ui_state,
                            requested_image_identifier,
                        );
                    }
                }
            }
        }
    });
}
