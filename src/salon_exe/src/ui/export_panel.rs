use eframe::{
    egui::{self, CollapsingHeader, ScrollArea, Ui},
    epaint::Color32,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use salon_core::{editor::GlobalEdit, runtime::Runtime, session::Session};

use super::{
    color_adjust, color_mixer, curve, effects, file_dialogues::file_dialogue_export_image, framing,
    histogram, light_adjust, masking, rotate_and_crop, widgets::EditorSlider, AppPage, AppUiState,
    EditorPanel,
};

pub fn export_panel(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    ui.spacing_mut().slider_width = ui.available_width() * 0.6;
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
        ui.label("File name ");
        ui.add(egui::TextEdit::singleline(
            ui_state.export_file_name.as_mut().unwrap(),
        ));
    });

    if ui_state.export_image_full_resolution.is_none() {
        ui_state.export_image_full_resolution = Some(session.editor.get_full_size_editted_image());
        ui_state.export_image_selected_resolution = ui_state.export_image_full_resolution.clone();
    }

    let full_resolution = ui_state
        .export_image_full_resolution
        .as_ref()
        .unwrap()
        .properties
        .dimensions
        .clone();
    let current_resolution = ui_state
        .export_image_selected_resolution
        .as_ref()
        .unwrap()
        .properties
        .dimensions
        .clone();
    let mut resolution = current_resolution.clone();
    let width_range = 1..=full_resolution.0;
    let height_range = 1..=full_resolution.1;

    ui.horizontal(|ui| {
        ui.label("Resolution ");
        ui.label("Width ");
        ui.add(egui::DragValue::new(&mut resolution.0).range(width_range));
        ui.label(" x ");
        ui.label("Height ");
        ui.add(egui::DragValue::new(&mut resolution.1).range(height_range));
    });

    if resolution != current_resolution {
        let new_resize_factor = if resolution.0 != current_resolution.0 {
            resolution.0 as f32 / full_resolution.0 as f32
        } else {
            resolution.1 as f32 / full_resolution.1 as f32
        };
        let new_image = session.toolbox.resize_image(
            ui_state
                .export_image_full_resolution
                .as_ref()
                .unwrap()
                .clone(),
            new_resize_factor,
        );
        ui_state.export_image_selected_resolution = Some(new_image);
    }

    if ui_state.export_quality.is_none() {
        ui_state.export_quality = Some(100);
    }

    ui.horizontal(|ui| {
        ui.label("Quality ");
        ui.add(
            EditorSlider::new(ui_state.export_quality.as_mut().unwrap(), 1..=100)
                .double_click_reset_value(100.0)
                .step_by(1.0),
        );
    });

    ui.horizontal(|ui| {
        if ui.button("Cancel").clicked() {
            exit_export_panel(ui_state);
        }
        ui.separator();
        if ui.button("Export").clicked() {
            file_dialogue_export_image(session, ui_state);
            exit_export_panel(ui_state);
        }
    });
}

pub fn exit_export_panel(ui_state: &mut AppUiState) {
    ui_state.app_page = AppPage::Editor;
    ui_state.export_quality = None;
    ui_state.export_file_name = None;
    ui_state.export_image_full_resolution = None;
    ui_state.export_image_selected_resolution = None;
}

fn editted_image_default_file_name(name: &String) -> String {
    let parts: Vec<&str> = name.rsplitn(2, '.').collect();
    if parts.len() == 2 {
        format!("{}_edit.{}", parts[1], parts[0])
    } else {
        format!("{}_edit", name)
    }
}
