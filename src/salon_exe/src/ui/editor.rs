use eframe::{
    egui::{self, CollapsingHeader, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{editor::GlobalEdit, session::Session};

use super::{color_adjust, curve, histogram, light_adjust, AppUiState, color_mixer, effects};

pub fn editor(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    let mut edit = session.editor.current_edit.clone();
    histogram(ui, session, ui_state);
    light_adjust(ui, session, ui_state, &mut edit.global);
    curve(ui, session, ui_state, &mut edit.global);
    color_adjust(ui, session, ui_state, &mut edit.global);
    color_mixer(ui, session, ui_state, &mut edit.global);
    effects(ui, session, ui_state, &mut edit.global);

    if session.state.current_image_index.is_none() {
        return;
    }
    if session.editor.current_edit != edit {
        session.editor.current_edit = edit;
        session.editor.execute_edit(&mut session.engine);
    }
}
