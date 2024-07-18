use eframe::{
    egui::{self, Align, CollapsingHeader, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::session::Session;

use super::{
    ui_set_current_editor_image,
    utils::get_max_image_size,
    widgets::{ThumbnailCallback, ThumbnailClip},
    AppPage, AppUiState,
};

pub fn library_side_panel(
    ctx: &egui::Context,
    ui: &mut Ui,
    session: &mut Session,
    ui_state: &mut AppUiState,
) {
    ui.horizontal(|ui| {
        let name = if let Some(album_index) = ui_state.selected_album {
            session.library.albums()[album_index].name.clone()
        } else {
            "All Photos".to_owned()
        };
        ui.label(name);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
            if ui
                .selectable_label(false, "🗖" /* emoji for "maximize", U+1F5D6 */)
                .on_hover_text("Return to Gallery")
                .clicked()
            {
                ui_state.app_page = AppPage::Library;
            }
        });
    });

    let bottom_y = ui.max_rect().max.y;
    let top_y = ui.max_rect().min.y;

    let cell_width = ui.available_width();
    let cell_height = cell_width;

    let table_height = ui.available_height();

    let mut table = TableBuilder::new(ui)
        .column(Column::exact(cell_width))
        .cell_layout(egui::Layout::centered_and_justified(
            egui::Direction::TopDown,
        ))
        .max_scroll_height(table_height);

    if let Some(ref requested_row) = ui_state.library_side_panel_requested_row.take() {
        table = table.scroll_to_row(*requested_row, Some(Align::Center));
    }

    let max_image_width = cell_height * 0.8;
    let max_image_height = max_image_width;

    let num_images = if let Some(album_index) = ui_state.selected_album {
        session.library.num_images_in_album(album_index)
    } else {
        session.library.num_images_total()
    };

    table.body(|mut body| {
        body.rows(cell_width, num_images, |mut row| {
            let row_index = row.index();
            let image_identifier = if let Some(album_index) = ui_state.selected_album {
                session
                    .library
                    .get_identifier_at_index_for_album(row_index, album_index)
                    .clone()
            } else {
                session.library.get_identifier_at_index(row_index).clone()
            };

            let mut selected = false;

            if let Some(editor_image) = session.editor.current_image_identifier() {
                if editor_image == image_identifier {
                    selected = true;
                    ui_state.library_side_panel_current_row = Some(row_index);
                }
            }

            row.col(|ui| {
                if let Some(image) = session
                    .library
                    .get_thumbnail_from_identifier(&image_identifier)
                {
                    let cell_max_rect = ui.max_rect();
                    let image_frame_rect = egui::Rect::from_center_size(
                        cell_max_rect.center(),
                        egui::Vec2::new(cell_height * 0.98, cell_width * 0.98),
                    );

                    let mut image_framing_color = egui::Color32::from_gray(40);
                    if selected {
                        image_framing_color = egui::Color32::from_gray(90);
                    } else {
                        if let Some(pos) = ui.input(|i| i.pointer.latest_pos()) {
                            if cell_max_rect.contains(pos) {
                                image_framing_color = egui::Color32::from_gray(60);
                            }
                        }
                    }

                    ui.painter().rect_filled(
                        image_frame_rect,
                        egui::Rounding::ZERO,
                        image_framing_color,
                    );

                    let aspect_ratio = image.aspect_ratio();
                    let size = get_max_image_size(aspect_ratio, max_image_width, max_image_height);
                    let (mut rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
                    let mut y_clip = ThumbnailClip::None;
                    if bottom_y < rect.max.y {
                        rect.max.y = bottom_y;
                        y_clip = ThumbnailClip::Bottom;
                    } else if top_y > rect.min.y {
                        rect.min.y = top_y;
                        y_clip = ThumbnailClip::Top
                    }
                    ui.centered_and_justified(|ui| {
                        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                            rect,
                            ThumbnailCallback {
                                image: image,
                                allocated_ui_rect: rect,
                                clip: y_clip,
                            },
                        ));
                    });
                    if response.clicked() {
                        ui_set_current_editor_image(ctx, session, ui_state, image_identifier);
                    }
                }
            });
        });
    });
}
