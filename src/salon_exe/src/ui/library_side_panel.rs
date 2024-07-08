use eframe::{
    egui::{self, CollapsingHeader, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::session::Session;

use super::{
    ui_set_current_editor_image,
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

    let max_height = ui.available_height();
    let mut table = TableBuilder::new(ui)
        .column(Column::auto())
        .cell_layout(egui::Layout::centered_and_justified(
            egui::Direction::TopDown,
        ))
        .max_scroll_height(max_height);
    let row_height = ui_state.last_frame_size.unwrap().1 * 0.1;
    let image_height = row_height * 0.8;

    let num_images = if let Some(album_index) = ui_state.selected_album {
        session.library.num_images_in_album(album_index)
    } else {
        session.library.num_images_total()
    };

    table.body(|mut body| {
        body.rows(row_height, num_images, |mut row| {
            let row_index = row.index();
            row.col(|ui| {
                let image_identifier = if let Some(album_index) = ui_state.selected_album {
                    session
                        .library
                        .get_identifier_at_index_for_album(row_index, album_index)
                        .clone()
                } else {
                    session.library.get_identifier_at_index(row_index).clone()
                };
                if let Some(image) = session
                    .library
                    .get_thumbnail_from_identifier(&image_identifier)
                {
                    let aspect_ratio = image.aspect_ratio();
                    let image_width = image_height * aspect_ratio;
                    let size = egui::Vec2 {
                        x: image_width,
                        y: image_height,
                    };
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
