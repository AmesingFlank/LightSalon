use num::Num;

use super::{
    mat::Mat2x2,
    rectangle::Rectangle,
    vec::{vec2, vec3, Vec2, Vec3, Vec4},
};

pub fn div_up(a: u32, b: u32) -> u32 {
    (a + b - 1) / b
}

pub fn mix(a: f32, b: f32, t: f32) -> f32 {
    return a * (1.0 - t) + b * t;
}

pub fn step(edge: f32, x: f32) -> f32 {
    if edge <= x {
        1.0
    } else {
        0.0
    }
}

pub fn clamp(x: f32, min: f32, max: f32) -> f32 {
    x.max(min).min(max)
}

pub fn get_rotation_mat(rotation_radians: f32) -> Mat2x2<f32> {
    Mat2x2::from_rows(
        vec2((rotation_radians.cos(), -rotation_radians.sin())),
        vec2((rotation_radians.sin(), rotation_radians.cos())),
    )
}

pub fn ray_ray_intersect(
    start_0: Vec2<f32>,
    dir_0: Vec2<f32>,
    start_1: Vec2<f32>,
    dir_1: Vec2<f32>,
) -> Option<(f32, f32)> {
    // https://stackoverflow.com/questions/563198/how-do-you-detect-where-two-line-segments-intersect/565282#565282
    let dir_cross = dir_0.cross(&dir_1);
    if dir_cross == 0.0 {
        return None;
    }
    let t0 = (start_1 - start_0).cross(&dir_1) / dir_0.cross(&dir_1);
    let t1 = (start_0 - start_1).cross(&dir_0) / dir_1.cross(&dir_0);
    Some((t0, t1))
}

pub fn ray_segment_intersect(
    ray_start: Vec2<f32>,
    ray_dir: Vec2<f32>,
    segment_start: Vec2<f32>,
    segment_end: Vec2<f32>,
) -> Option<f32> {
    let seg_dir = (segment_end - segment_start).normalized();
    let seg_len = (segment_end - segment_start).length();
    let (ray_t, seg_t) = ray_ray_intersect(ray_start, ray_dir, segment_start, seg_dir)?;
    if seg_t < 0.0 || seg_t > seg_len {
        return None;
    }
    Some(ray_t)
}

pub fn legalize_crop_rect(
    rotation_degrees: f32,
    crop_rect: Rectangle,
    image_aspect_ratio: f32,
) -> Option<Rectangle> {
    let rotation_radians = rotation_degrees.to_radians();
    let rotation_mat = get_rotation_mat(rotation_radians);
    let mut corners = vec![
        vec2((0.0, 0.0)),
        vec2((0.0, 1.0)),
        vec2((1.0, 1.0)),
        vec2((1.0, 0.0)),
    ];
    let mut crop_rect_center = crop_rect.center;
    crop_rect_center.x /= image_aspect_ratio;
    for corner in corners.iter_mut() {
        corner.x /= image_aspect_ratio;
        *corner = *corner - crop_rect_center;
        *corner = rotation_mat * *corner;
    }
    let segments = vec![
        (corners[0], corners[1]),
        (corners[1], corners[2]),
        (corners[2], corners[3]),
        (corners[3], corners[0]),
    ];
    let mut crop_rect_corners = vec![
        crop_rect.min(),
        vec2((crop_rect.min().x, crop_rect.max().y)),
        crop_rect.max(),
        vec2((crop_rect.max().x, crop_rect.min().y)),
    ];
    for corner in crop_rect_corners.iter_mut() {
        corner.x /= image_aspect_ratio;
        *corner = *corner - crop_rect_center;
    }

    let mut new_rect = None;
    for corner in crop_rect_corners.iter() {
        let ray_start = vec2((0.0, 0.0));
        let ray_dir = corner.normalized();
        let current_dist_to_corner = corner.length();
        for seg in segments.iter() {
            let t = ray_segment_intersect(ray_start, ray_dir, seg.0, seg.1);
            if let Some(t) = t {
                if t >= 0.0 && t < current_dist_to_corner {
                    if new_rect.is_none() {
                        new_rect = Some(crop_rect);
                        new_rect.as_mut().unwrap().size.x /= image_aspect_ratio;
                    }
                    new_rect.as_mut().unwrap().size.x = new_rect
                        .as_mut()
                        .unwrap()
                        .size
                        .x
                        .min((t * ray_dir.x * 2.0).abs());
                    new_rect.as_mut().unwrap().size.y = new_rect
                        .as_mut()
                        .unwrap()
                        .size
                        .y
                        .min((t * ray_dir.y * 2.0).abs());
                }
            }
        }
    }

    let mut new_rect = new_rect?;
    new_rect.size.x *= image_aspect_ratio;
    Some(new_rect)
}
