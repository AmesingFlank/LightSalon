use eframe::{
    egui::{self, CollapsingHeader, Ui},
    egui_wgpu,
};
use salon_core::{library::LibraryImageIdentifier, session::Session};

use super::{
    bottom_bar, editor, export_panel::export_panel, keyboard_response, library_albums_browser,
    library_images_browser, library_side_panel, main_image, menu_bar, AppPage, AppUiState,
};

pub fn app_ui(ctx: &egui::Context, session: &mut Session, ui_state: &mut AppUiState) {
    let last_frame_size = ui_state
        .last_frame_size
        .expect("expecting a last frame size");

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        menu_bar(ui, session, ui_state);
    });
    egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
        bottom_bar(ui, session, ui_state);
    });

    match ui_state.app_page {
        AppPage::Library => {
            egui::SidePanel::left("albums_browser_panel")
                .min_width(last_frame_size.0 * 0.1)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.set_width(ui.available_width());
                    library_albums_browser(ui, session, ui_state);
                });
            egui::CentralPanel::default().show(ctx, |ui| {
                library_images_browser(ui, session, ui_state);
            });
        }
        AppPage::Editor => {
            egui::SidePanel::left("library_panel")
                .default_width(last_frame_size.0 * 0.1)
                .resizable(false)
                .show(ctx, |ui| {
                    // ui.set_width(ui.available_width());
                    library_side_panel(ui, session, ui_state);
                });
            egui::SidePanel::right("editor_panel")
                .default_width(last_frame_size.0 * 0.2)
                .max_width(last_frame_size.0 * 0.2)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.set_width(ui.available_width());
                    editor(ui, session, ui_state);
                });
            egui::CentralPanel::default().show(ctx, |ui| {
                main_image(ui, session, ui_state);
            });
        }
        AppPage::Export => {
            egui::SidePanel::right("export_panel")
                .default_width(last_frame_size.0 * 0.2)
                .max_width(last_frame_size.0 * 0.2)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.set_width(ui.available_width());
                    export_panel(ui, session, ui_state);
                });
            egui::CentralPanel::default().show(ctx, |ui| {
                main_image(ui, session, ui_state);
            });
        }
    }
    keyboard_response(ctx, session, ui_state);
}

pub fn ui_set_current_editor_image(
    session: &mut Session,
    ui_state: &mut AppUiState,
    identifier: LibraryImageIdentifier,
) {
    ui_state.main_image_select_error_msg = session.set_current_image(identifier).err();
    ui_state.reset_for_different_image();
}
