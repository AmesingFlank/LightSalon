use eframe::{
    egui::{self, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::{library::AddImageResult, session::Session};

use super::{file_menu, edit_menu, AppUiState};

pub fn menu_bar(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    ui.horizontal_wrapped(|ui| {
        file_menu(ui, session, ui_state);
        edit_menu(ui, session, ui_state);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
            ui.checkbox(&mut ui_state.show_grid, "Show Grid");
        });
    });
}
