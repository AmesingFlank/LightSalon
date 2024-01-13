use eframe::{
    egui::{self, Ui},
    epaint::Pos2,
};
use salon_core::{
    runtime::Image,
    utils::vec::{vec2, Vec2},
};

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

pub fn pos2_to_vec2(p: Pos2) -> Vec2<f32> {
    vec2((p.x, p.y))
}
