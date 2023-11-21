use eframe::{
    egui::{self, CollapsingHeader, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{editor::GlobalEdit, session::Session};

use super::{
    color_adjust, color_mixer, curve, effects, histogram, light_adjust, AppUiState, EditorPanel,
};

pub fn editor(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    ui.horizontal(|ui| {
        ui.selectable_value(
            &mut ui_state.editor_panel,
            EditorPanel::LightAndColor,
            "Adjust",
        );
        ui.selectable_value(
            &mut ui_state.editor_panel,
            EditorPanel::CropAndRotate,
            "Crop",
        );
    });

    ui.separator();

    let mut edit = session.editor.current_edit.clone();

    match ui_state.editor_panel {
        EditorPanel::LightAndColor => {
            histogram(ui, session, ui_state);
            light_adjust(ui, session, ui_state, &mut edit.global);
            curve(ui, session, ui_state, &mut edit.global);
            color_adjust(ui, session, ui_state, &mut edit.global);
            color_mixer(ui, session, ui_state, &mut edit.global);
            effects(ui, session, ui_state, &mut edit.global);
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
