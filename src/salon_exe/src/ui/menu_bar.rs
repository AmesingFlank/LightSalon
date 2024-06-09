use eframe::{
    egui::{self, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::session::Session;

use super::{edit_menu, file_menu, AppPage, AppUiState};

pub fn menu_bar(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    ui.horizontal_wrapped(|ui| match ui_state.app_page {
        AppPage::Library => {}
        AppPage::Editor => {
            file_menu(ui, session, ui_state);
            edit_menu(ui, session, ui_state);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                ui.checkbox(&mut ui_state.show_grid, "Show Grid");
                ui.separator();
                ui.checkbox(&mut ui_state.show_comparison, "Show Comparison");
            });
        }
    });
}
