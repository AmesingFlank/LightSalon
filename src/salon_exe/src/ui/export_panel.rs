use eframe::{
    egui::{self, CollapsingHeader, ScrollArea, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{editor::GlobalEdit, runtime::Runtime, session::Session};

use super::{
    color_adjust, color_mixer, curve, effects, file_dialogues::file_dialogue_export_image, framing,
    histogram, light_adjust, masking, rotate_and_crop, AppPage, AppUiState, EditorPanel,
};

pub fn export_panel(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    if ui_state.export_file_name.is_none() {
        if let Some(ref name) = session
            .library
            .get_metadata(&session.editor.current_image_identifier().unwrap())
            .name
        {
            ui_state.export_file_name = Some(editted_image_default_file_name(name))
        } else {
            ui_state.export_file_name = Some("output.jpg".to_owned());
        }
    }

    ui.horizontal(|ui| {
        ui.label("File name: ");
        ui.add(egui::TextEdit::singleline(
            ui_state.export_file_name.as_mut().unwrap(),
        ));
    });

    // ui.separator();

    // ui.horizontal(|ui| {
    //     ui.label("Resolution: ");
    //     ui.label("Width ");
    //     let clamp_range = 1..=Runtime::get_required_max_texture_dim_1d_2d();
    //     ui.add(
    //         egui::DragValue::new(&mut ui_state.crop_rect_aspect_ratio.0).range(clamp_range.clone()),
    //     );
    //     ui.label(" x ");
    //     ui.label("Height ");
    //     ui.add(
    //         egui::DragValue::new(&mut ui_state.crop_rect_aspect_ratio.1).range(clamp_range.clone()),
    //     );
    // });

    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Cancel").clicked() {
            exit_export_panel(ui_state);
        }
        ui.separator();
        if ui.button("Export").clicked() {
            file_dialogue_export_image(ui.ctx().clone(), session, ui_state);
            exit_export_panel(ui_state);
        }
    });
}

pub fn exit_export_panel(ui_state: &mut AppUiState) {
    ui_state.app_page = AppPage::Editor;
    ui_state.export_file_name = None;
}

fn editted_image_default_file_name(name: &String) -> String {
    let parts: Vec<&str> = name.rsplitn(2, '.').collect();
    if parts.len() == 2 {
        format!("{}_edit.{}", parts[1], parts[0])
    } else {
        format!("{}_edit", name)
    }
}
