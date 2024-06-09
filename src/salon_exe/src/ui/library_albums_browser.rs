use eframe::{
    egui::{self, Ui},
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
}
