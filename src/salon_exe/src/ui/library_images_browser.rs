use eframe::{
    egui::{self, Ui},
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
    let table = TableBuilder::new(ui)
        .cell_layout(egui::Layout::centered_and_justified(
            egui::Direction::TopDown,
        ))
        .max_scroll_height(max_height)
        .columns(Column::exact(column_width), num_columns);

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
                            ui_state.app_page = AppPage::Editor;
                            ui_set_current_editor_image(ctx, session, ui_state, image_identifier);
                        }
                    }
                });
            }
        });
    });
}
