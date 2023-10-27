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

    if session.current_image_index.is_none() {
        return;
    }
    if session.editor.current_state != editor_state {
        session.editor.current_state = editor_state;
        let module = session.editor.current_state.to_ir_module();
        let input_image_index = session.current_image_index.unwrap();
        let input_image = session.library.get_image(input_image_index);
        let result = session.engine.execute_module(&module, input_image);
        session.current_process_result = Some(result)
    }
}
