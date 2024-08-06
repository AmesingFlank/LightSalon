use eframe::{
    egui::{self, CursorIcon, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::{
    library::{ImageRating, LibraryImageIdentifier},
    session::Session,
};

use super::{
    ui_set_current_editor_image,
    utils::{get_album_name_text_with_emoji_and_count, get_max_image_size},
    widgets::{ThumbnailCallback, ThumbnailClip},
    AppPage, AppUiState,
};

pub fn library_images_browser(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    let bottom_y = ui.max_rect().max.y;
    let top_y = ui.max_rect().min.y;

    let max_height = ui.available_height();
    let num_columns = (ui.available_width() as usize / 200).max(1).min(6);
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
    let max_image_height = row_height * 0.8; // leave some room for the rating stars
    let max_image_width = column_width * 0.9;
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
                        let cell_max_rect = ui.max_rect();
                        let image_frame_rect = egui::Rect::from_center_size(
                            cell_max_rect.center(),
                            egui::Vec2::new(
                                cell_max_rect.width() * 1.0,
                                // somehow, only this makes the vertical gaps and horizontal gaps similar in size..
                                cell_max_rect.height() * 0.98,
                            ),
                        );

                        let image_frame_response =
                            ui.allocate_rect(image_frame_rect, egui::Sense::click());
                        let image_frame_clicked = image_frame_response.clicked();
                        // cannot use `image_frame_response.hovered()` here, because it is false when hovering over stars rating text
                        let image_frame_hovered = ui.input(|i| {
                            if let Some(pos) = i.pointer.latest_pos() {
                                return image_frame_rect.contains(pos);
                            }
                            false
                        });

                        let mut image_framing_color = egui::Color32::from_gray(40);

                        if image_frame_hovered {
                            image_framing_color = egui::Color32::from_gray(60);
                        }
                        ui.painter().rect_filled(
                            image_frame_rect,
                            egui::Rounding::ZERO,
                            image_framing_color,
                        );

                        if image_frame_hovered {
                            let metadata = session.library.get_metadata(&image_identifier);
                            if let Some(name) = metadata.name {
                                let image_name_rect = egui::Rect::from_min_max(
                                    cell_max_rect.min,
                                    egui::Pos2::new(
                                        cell_max_rect.max.x,
                                        cell_max_rect.min.y + row_height * 0.1,
                                    ),
                                );

                                ui.allocate_ui_at_rect(image_name_rect, |ui| {
                                    ui.centered_and_justified(|ui| ui.label(name))
                                });
                            }
                        }

                        let aspect_ratio = image.aspect_ratio();
                        let image_size =
                            get_max_image_size(aspect_ratio, max_image_width, max_image_height);
                        let mut image_rect =
                            egui::Rect::from_center_size(image_frame_rect.center(), image_size);
                        let mut y_clip = ThumbnailClip::None;
                        if bottom_y < image_rect.max.y {
                            image_rect.max.y = bottom_y;
                            y_clip = ThumbnailClip::Bottom;
                        } else if top_y > image_rect.min.y {
                            image_rect.min.y = top_y;
                            y_clip = ThumbnailClip::Top
                        }

                        ui.centered_and_justified(|ui| {
                            ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                                image_rect,
                                ThumbnailCallback {
                                    image: image,
                                    allocated_ui_rect: image_rect,
                                    clip: y_clip,
                                },
                            ));
                        });

                        let rating_rect = egui::Rect::from_min_max(
                            egui::Pos2::new(
                                cell_max_rect.min.x,
                                cell_max_rect.min.y + row_height * 0.9,
                            ),
                            cell_max_rect.max,
                        );
                        let rating_clicked = image_rating(
                            ui,
                            session,
                            ui_state,
                            rating_rect,
                            image_frame_hovered,
                            &image_identifier,
                        );

                        if image_frame_response.hovered() {
                            ui.output_mut(|out| out.cursor_icon = CursorIcon::PointingHand);
                        }

                        if image_frame_clicked && !rating_clicked {
                            ui_state.app_page = AppPage::Editor;
                            ui_state.library_side_panel_requested_row = Some(image_index);
                            ui_state.library_images_browser_requested_row = Some(row_index);
                            ui_set_current_editor_image(
                                session,
                                ui_state,
                                image_identifier.clone(),
                            );
                        }
                        image_frame_response.context_menu(|ui| {
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

fn image_rating(
    ui: &mut Ui,
    session: &mut Session,
    _ui_state: &mut AppUiState,
    rating_rect: egui::Rect,
    image_frame_hovered: bool,
    identifier: &LibraryImageIdentifier,
) -> bool {
    let mut num_stars = 0;
    let original_rating = session.library.get_rating(&identifier);
    if let Some(rated_stars) = original_rating.num_stars {
        num_stars = rated_stars;
    }

    if !image_frame_hovered && original_rating.num_stars.is_none() {
        return false;
    }

    let mut clicked_rating = false;

    let mut star_rects = Vec::new();

    for i in 0..ImageRating::MAX_STARS {
        let delta_from_center = i as f32 - (ImageRating::MAX_STARS / 2) as f32;
        let star_rect = egui::Rect::from_center_size(
            egui::Pos2::new(
                rating_rect.center().x + delta_from_center * rating_rect.width() * 0.1,
                rating_rect.center().y,
            ),
            egui::Vec2::new(rating_rect.width() * 0.1, rating_rect.height()),
        );
        if let Some(pos) = ui.input(|i| i.pointer.latest_pos()) {
            if star_rect.contains(pos) {
                ui.output_mut(|out| out.cursor_icon = CursorIcon::PointingHand);
                num_stars = i + 1;
            }
        }
        star_rects.push(star_rect);
    }

    for i in 0..ImageRating::MAX_STARS {
        let star_rect = star_rects[i as usize];
        ui.allocate_ui_at_rect(star_rect, |ui| {
            let star_text = if i < num_stars { "★" } else { "☆" };

            let label_response = ui.add(
                egui::Label::new(star_text)
                    .selectable(false)
                    .sense(egui::Sense::click()),
            );

            if label_response.clicked() {
                let selected_num_stars = i + 1;
                let selected_rating = ImageRating::new(Some(selected_num_stars));
                if selected_rating == original_rating {
                    session
                        .library
                        .set_rating(&identifier, ImageRating::new(None));
                } else {
                    session.library.set_rating(&identifier, selected_rating);
                }
                clicked_rating = true;
            }
        });
    }
    clicked_rating
}
