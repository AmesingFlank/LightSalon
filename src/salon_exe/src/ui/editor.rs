use eframe::{
    egui::{self, CollapsingHeader, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{editor::EditorState, session::Session};

use super::{color_adjust, curve, histogram, light_adjust, AppUiState};

pub fn editor(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    let mut editor_state = session.editor.current_state.clone();
    histogram(ui, session, ui_state);
    light_adjust(ui, session, ui_state, &mut editor_state);
    color_adjust(ui, session, ui_state, &mut editor_state);
    curve(ui, session, ui_state, &mut editor_state);

    if session.state.current_image_index.is_none() {
        return;
    }
    if session.editor.current_state != editor_state {
        session.editor.current_state = editor_state;
        session.execute_edit();
    }
}
