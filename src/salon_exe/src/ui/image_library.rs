use eframe::{
    egui::{self, Ui},
    egui_wgpu,
};
use egui_extras::{Column, TableBuilder};
use salon_core::session::Session;

use super::{widgets::ThumbnailCallback, AppUiState};

pub fn image_library(ui: &mut Ui, session: &mut Session, ui_state: &mut AppUiState) {
    let mut table = TableBuilder::new(ui).column(Column::auto()).cell_layout(
        egui::Layout::centered_and_justified(egui::Direction::TopDown),
    );
    let row_height = ui_state.last_frame_size.unwrap().1 * 0.1;
    let image_height = row_height * 0.8;
    table.body(|mut body| {
        body.rows(
            row_height,
            session.library.num_images() as usize,
            |mut row| {
                let row_index = row.index();
                row.col(|ui| {
                    let image = session.library.get_thumbnail(row_index);
                    let aspect_ratio = image.aspect_ratio();
                    let image_width = image_height / aspect_ratio;
                    let size = egui::Vec2 {
                        x: image_width,
                        y: image_height,
                    };
                    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
                    ui.centered_and_justified(|ui| {
                        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                            rect,
                            ThumbnailCallback { image: image },
                        ));
                    });
                    if response.clicked() {
                        session.set_current_image(row_index);
                        ui_state.reset_for_different_image();
                    }
                });
            },
        );
    });
}
