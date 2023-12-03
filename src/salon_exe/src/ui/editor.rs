use eframe::{
    egui::{self, CollapsingHeader, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{editor::GlobalEdit, session::Session};

use super::{
    color_adjust, color_mixer, curve, effects, histogram, light_adjust, masking, AppUiState,
    EditorPanel,
};

pub fn editor(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    ui.horizontal(|ui| {
        let response = ui.selectable_value(
            &mut ui_state.editor_panel,
            EditorPanel::LightAndColor,
            "Light and Color",
        );
        if response.clicked() {
            session.editor.execute_edit(&mut session.engine);
        }
        ui.separator();
        ui.selectable_value(
            &mut ui_state.editor_panel,
            EditorPanel::CropAndRotate,
            "Crop and Rotate",
        );
    });

    ui.separator();

    let mut edit = session.editor.current_edit.clone();
    let global_edit = &mut edit.masked_edits[ui_state.selected_mask_index].edit;

    match ui_state.editor_panel {
        EditorPanel::LightAndColor => {
            histogram(ui, session, ui_state);
            ui.separator();
            masking(ui, session, ui_state);
            light_adjust(ui, session, ui_state, global_edit);
            curve(ui, session, ui_state, global_edit);
            color_adjust(ui, session, ui_state, global_edit);
            color_mixer(ui, session, ui_state, global_edit);
            effects(ui, session, ui_state, global_edit);
        }
        EditorPanel::CropAndRotate => {}
    }

    if session.state.current_image_index.is_none() {
        return;
    }
    if session.editor.current_edit != edit {
        session.editor.current_edit = edit;
        session.editor.execute_edit(&mut session.engine);
    }
}
