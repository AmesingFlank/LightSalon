use eframe::egui::{Ui, self};
use salon_core::runtime::Image;

pub fn get_image_size_in_ui(ui: &Ui, image: &Image) -> egui::Vec2 {
    let max_x = ui.available_width();
    let max_y = ui.available_height();
    let ui_aspect_ratio = max_y / max_x;

    let image_aspect_ratio = image.aspect_ratio();

    let size = if image_aspect_ratio >= ui_aspect_ratio {
        egui::Vec2 {
            x: max_y / image_aspect_ratio,
            y: max_y,
        }
    } else {
        egui::Vec2 {
            x: max_x,
            y: max_x * image_aspect_ratio,
        }
    };
    size
}
