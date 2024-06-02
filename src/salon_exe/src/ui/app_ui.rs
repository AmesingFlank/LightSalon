use eframe::{
    egui::{self, CollapsingHeader, Ui},
    egui_wgpu,
};
use salon_core::{library::LibraryImageIdentifier, session::Session};

use super::{
    bottom_bar, editor, image_library, keyboard_response, main_image, menu_bar, AppUiState,
};

pub fn app_ui(ctx: &egui::Context, session: &mut Session, ui_state: &mut AppUiState) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        menu_bar(ui, session, ui_state);
    });
    egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
        bottom_bar(ui, session, ui_state);
    });

    let last_frame_size = ui_state
        .last_frame_size
        .expect("expecting a last frame size");
    egui::SidePanel::left("library_panel")
        .default_width(last_frame_size.0 * 0.2)
        .resizable(true)
        .show(ctx, |ui| {
            // ui.set_width(ui.available_width());
            image_library(ctx, ui, session, ui_state);
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
        main_image(ctx, ui, session, ui_state);
        keyboard_response(ui, session, ui_state);
    });
}

pub fn ui_set_current_image(
    ctx: &egui::Context,
    session: &mut Session,
    ui_state: &mut AppUiState,
    identifier: LibraryImageIdentifier,
) {
    ui_state.main_image_select_error_msg = session.set_current_image(identifier).err();
    ctx.request_repaint();
    ui_state.reset_for_different_image();
}
