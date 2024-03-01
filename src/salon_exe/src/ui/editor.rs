use eframe::{
    egui::{self, CollapsingHeader, ScrollArea, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{editor::GlobalEdit, session::Session};

use super::{
    color_adjust, color_mixer, curve, effects, histogram, light_adjust, masking, AppUiState,
    EditorPanel,
};

pub fn editor(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    if session.editor.current_edit_context_ref().is_none() {
        return;
    }
    ui.horizontal(|ui| {
        let response = ui.selectable_value(
            &mut ui_state.editor_panel,
            EditorPanel::LightAndColor,
            "Light and Color",
        );
        if response.clicked() {
            session.editor.commit_transient_edit(true);
        }
        ui.separator();
        ui.selectable_value(
            &mut ui_state.editor_panel,
            EditorPanel::CropAndRotate,
            "Crop and Rotate",
        );
    });

    ui.separator();

    match ui_state.editor_panel {
        EditorPanel::LightAndColor => {
            let mut transient_edit = session
                .editor
                .current_edit_context_ref()
                .unwrap()
                .transient_edit_ref()
                .clone();

            histogram(ui, session, ui_state);
            ui.separator();
            ScrollArea::vertical().show(ui, |ui| {
                masking(ui, session, ui_state, &mut transient_edit);

                let global_edit: &mut GlobalEdit =
                    &mut transient_edit.masked_edits[ui_state.selected_mask_index].edit;

                light_adjust(ui, session, ui_state, global_edit);
                curve(ui, session, ui_state, global_edit);
                color_adjust(ui, session, ui_state, global_edit);
                color_mixer(ui, session, ui_state, global_edit);
                effects(ui, session, ui_state, global_edit);
            });

            ui.input(|i| {
                session.editor.update_transient_edit(transient_edit, true);
                if !i.pointer.any_down() {
                    session.editor.commit_transient_edit(false);
                }
                // else a slider could still be being dragged, so the edit should remain transient
            });
        }
        EditorPanel::CropAndRotate => {}
    }
}
