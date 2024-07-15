use eframe::{
    egui::{self, CursorIcon, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::{library::LibraryImageIdentifier, session::Session};

use super::{
    ui_set_current_editor_image,
    utils::{get_album_name_text_with_emoji_and_count, get_max_image_size},
    widgets::{ThumbnailCallback, ThumbnailClip},
    AppPage, AppUiState,
};

pub fn library_images_browser(
    ctx: &egui::Context,
    ui: &mut Ui,
    session: &mut Session,
    ui_state: &mut AppUiState,
) {
    let bottom_y = ui.max_rect().max.y;
    let top_y = ui.max_rect().min.y;

    let max_height = ui.available_height();
    let num_columns = 6;
    let column_width = (ui.available_width() / num_columns as f32) * 0.95;
    let mut table = TableBuilder::new(ui)
        .cell_layout(egui::Layout::centered_and_justified(
            egui::Direction::TopDown,
        ))
        .max_scroll_height(max_height)
        .columns(Column::exact(column_width), num_columns);
    if let Some(requested_row) = ui_state.library_images_browser_requested_row.take() {
        table = table.scroll_to_row(requested_row, None);
    }

    let row_height = column_width;
    let max_image_height = row_height * 0.9;
    let max_image_width = max_image_height;
    let num_images = if let Some(album_index) = ui_state.selected_album {
        session.library.num_images_in_album(album_index)
    } else {
        session.library.num_images_total()
    };
    let mut num_rows = num_images / num_columns;
    if num_images % num_columns != 0 {
        num_rows = num_rows + 1;
    }

    let mut removed_image = None;

    table.body(|body| {
        body.rows(row_height, num_rows, |mut row| {
            let row_index = row.index();
            for i in 0..num_columns {
                let image_index = row_index * num_columns + i;
                row.col(|ui| {
                    if image_index >= num_images {
                        return;
                    }
                    let image_identifier = if let Some(album_index) = ui_state.selected_album {
                        session
                            .library
                            .get_identifier_at_index_for_album(image_index, album_index)
                            .clone()
                    } else {
                        session.library.get_identifier_at_index(image_index).clone()
                    };
                    if let Some(image) = session
                        .library
                        .get_thumbnail_from_identifier(&image_identifier)
                    {
                        let aspect_ratio = image.aspect_ratio();
                        let image_size =
                            get_max_image_size(aspect_ratio, max_image_width, max_image_height);
                        let (mut rect, response) =
                            ui.allocate_exact_size(image_size, egui::Sense::click());
                        let mut y_clip = ThumbnailClip::None;
                        if bottom_y < rect.max.y {
                            rect.max.y = bottom_y;
                            y_clip = ThumbnailClip::Bottom;
                        } else if top_y > rect.min.y {
                            rect.min.y = top_y;
                            y_clip = ThumbnailClip::Top
                        }

                        let cell_max_rect = ui.max_rect();
                        let image_frame_rect = egui::Rect::from_center_size(
                            cell_max_rect.center(),
                            egui::Vec2::new(
                                cell_max_rect.width() * 1.0,
                                // somehow, only this makes the vertical gaps and horizontal gaps similar in size..
                                cell_max_rect.height() * 0.98,
                            ),
                        );
                        let mut image_framing_color = egui::Color32::from_gray(40);
                        if response.hovered() {
                            image_framing_color = egui::Color32::from_gray(60);
                        }
                        ui.painter().rect_filled(
                            image_frame_rect,
                            egui::Rounding::ZERO,
                            image_framing_color,
                        );

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

                        if response.hovered() {
                            ui.output_mut(|out| out.cursor_icon = CursorIcon::PointingHand);
                        }
                        if response.clicked() {
                            ui_state.app_page = AppPage::Editor;
                            ui_state.library_side_panel_requested_row = Some(image_index);
                            ui_state.library_images_browser_requested_row = Some(row_index);
                            ui_set_current_editor_image(
                                ctx,
                                session,
                                ui_state,
                                image_identifier.clone(),
                            );
                        }
                        response.context_menu(|ui| {
                            ui.menu_button("Add to album", |ui| {
                                for i in 0..session.library.albums().len() {
                                    let can_add = !session.library.albums()[i]
                                        .contains_image(&image_identifier);
                                    let album_name_text = get_album_name_text_with_emoji_and_count(
                                        &session.library.albums()[i],
                                    );
                                    if ui
                                        .add_enabled(can_add, egui::Button::new(album_name_text))
                                        .clicked()
                                    {
                                        ui.close_menu();
                                        session
                                            .library
                                            .add_existing_item_into_album(&image_identifier, i);
                                    }
                                }
                            });
                            if let Some(album_index) = ui_state.selected_album {
                                if session.library.albums()[album_index]
                                    .contains_additional_image(&image_identifier)
                                {
                                    let remove_text = "Remove from album ".to_owned()
                                        + session.library.albums()[album_index].name.as_str();
                                    if ui.button(remove_text).clicked() {
                                        removed_image = Some(image_identifier.clone());
                                    }
                                }
                            }
                        });
                    }
                });
            }
        });
    });

    if let Some(removed_image) = removed_image {
        session
            .library
            .remove_image_from_album(ui_state.selected_album.clone().unwrap(), &removed_image);
    }
}
