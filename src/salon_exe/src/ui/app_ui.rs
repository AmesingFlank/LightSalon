use eframe::{
    egui::{self, CollapsingHeader, Ui},
    egui_wgpu,
};
use salon_core::session::Session;

use super::{bottom_bar, editor, file_menu, image_library, main_image, AppUiState};

pub fn app_ui(ctx: &egui::Context, session: &mut Session, ui_state: &mut AppUiState) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
            file_menu(ui, session, ui_state);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                ui.checkbox(&mut ui_state.show_grid, "Show Grid");
            })
        });
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
            image_library(ui, session, ui_state);
        });
    egui::SidePanel::right("editor_panel")
        .default_width(last_frame_size.0 * 0.2)
        .resizable(true)
        .show(ctx, |ui| {
            ui.set_width(ui.available_width());
            editor(ui, session, ui_state);
        });
    egui::CentralPanel::default().show(ctx, |ui| {
        main_image(ctx, ui, session, ui_state);
    });
}
