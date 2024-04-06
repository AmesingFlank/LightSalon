use num::{complex::ComplexFloat, Num};

use crate::{editor::Edit, ir::MaskPrimitive, session::Session};

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

pub fn get_rotation_mat_from_degrees(rotation_degrees: f32) -> Mat2x2<f32> {
    // this avoid some numerical precision issues in get_crop_rect_translation_bounds
    if (rotation_degrees % 360.0).abs() == 180.0 {
        return Mat2x2::from_rows(vec2((-1.0, 0.0)), vec2((0.0, -1.0)));
    }
    if (rotation_degrees % 360.0) == 90.0 || (rotation_degrees % 360.0) == -270.0 {
        return Mat2x2::from_rows(vec2((0.0, -1.0)), vec2((1.0, 0.0)));
    }
    if (rotation_degrees % 360.0) == -90.0 || (rotation_degrees % 360.0) == 270.0 {
        return Mat2x2::from_rows(vec2((0.0, 1.0)), vec2((-1.0, 0.0)));
    }
    get_rotation_mat(rotation_degrees.to_radians())
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

// https://github.com/AmesingFlank/OxfordCSNotes/blob/master/GMOD18-19/Lecture1_points%2C%20line%2Cline%20segments.%20relative%20positions%2C%20polyline.pdf
pub fn counter_clockwise_triangle_area(p0: Vec2<f32>, p1: Vec2<f32>, p2: Vec2<f32>) -> f32 {
    ((p1.x - p0.x) * (p2.y - p0.y) - (p2.x - p0.x) * (p1.y - p0.y)) * 0.5
}

pub fn point_is_left_of_segment(
    point: Vec2<f32>,
    segment_start: Vec2<f32>,
    segment_end: Vec2<f32>,
) -> bool {
    let area = counter_clockwise_triangle_area(segment_start, segment_end, point);
    area > 0.0
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

fn get_full_image_corner_positions(
    rotation_degrees: f32,
    crop_rect: Rectangle,
    image_aspect_ratio: f32,
) -> Vec<Vec2<f32>> {
    let rotation_mat = get_rotation_mat_from_degrees(rotation_degrees);
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
    corners
}

fn get_full_image_edge_segments(
    rotation_degrees: f32,
    crop_rect: Rectangle,
    image_aspect_ratio: f32,
) -> [(Vec2<f32>, Vec2<f32>); 4] {
    let corners = get_full_image_corner_positions(rotation_degrees, crop_rect, image_aspect_ratio);
    [
        (corners[0], corners[1]),
        (corners[1], corners[2]),
        (corners[2], corners[3]),
        (corners[3], corners[0]),
    ]
}

fn get_crop_rect_corner_positions(crop_rect: Rectangle, image_aspect_ratio: f32) -> [Vec2<f32>; 4] {
    let mut crop_rect_center = crop_rect.center;
    crop_rect_center.x /= image_aspect_ratio;

    let mut crop_rect_corners = [
        crop_rect.min(),
        vec2((crop_rect.min().x, crop_rect.max().y)),
        crop_rect.max(),
        vec2((crop_rect.max().x, crop_rect.min().y)),
    ];

    for corner in crop_rect_corners.iter_mut() {
        corner.x /= image_aspect_ratio;
        *corner = *corner - crop_rect_center;
    }

    crop_rect_corners
}

pub fn maybe_shrink_crop_rect_due_to_rotation(
    rotation_degrees: f32,
    crop_rect: Rectangle,
    image_aspect_ratio: f32,
) -> Option<Rectangle> {
    let full_image_edge_segments =
        get_full_image_edge_segments(rotation_degrees, crop_rect, image_aspect_ratio);
    let crop_rect_corners = get_crop_rect_corner_positions(crop_rect, image_aspect_ratio);

    let mut new_rect = None;
    for corner in crop_rect_corners.iter() {
        let ray_start = vec2((0.0, 0.0));
        let ray_dir = corner.normalized();
        let current_dist_to_corner = corner.length();
        for seg in full_image_edge_segments.iter() {
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

fn update_translation_bounds(
    full_image_edge_segments: [(Vec2<f32>, Vec2<f32>); 4],
    point_in_image: Vec2<f32>,
    translation_dir: Vec2<f32>,
    curr_bounds: (f32, f32),
) -> (f32, f32) {
    let mut bounds = curr_bounds;
    for seg in full_image_edge_segments.iter() {
        let t = ray_segment_intersect(point_in_image, translation_dir, seg.0, seg.1);
        if let Some(t) = t {
            let is_outside_full_image = point_is_left_of_segment(point_in_image, seg.0, seg.1);
            if is_outside_full_image {
                // this corner is already on the wrong side
                if t >= 0.0 {
                    // make sure to push it back
                    bounds.0 = bounds.0.max(t)
                } else {
                    bounds.1 = bounds.1.min(t)
                }
            } else {
                if t > 0.0 {
                    // don't go past the segment
                    bounds.1 = bounds.1.min(t)
                } else if t == 0.0 {
                    // colinear
                    let can_go_forwards =
                        !point_is_left_of_segment(point_in_image + translation_dir, seg.0, seg.1);
                    if can_go_forwards {
                        // don't go backwards
                        bounds.0 = bounds.0.max(0.0)
                    } else {
                        // don't go backwards
                        bounds.1 = bounds.1.min(0.0)
                    }
                } else {
                    bounds.0 = bounds.0.max(t)
                }
            }
        }
    }
    bounds
}

// bounds in ui-reference frame (crop-rect is non-rotated, full image is rotated, x and y scale are both relative to full image height)
pub fn get_crop_rect_translation_bounds(
    rotation_degrees: f32,
    crop_rect: Rectangle,
    image_aspect_ratio: f32,
) -> [(f32, f32); 4] {
    let full_image_edge_segments =
        get_full_image_edge_segments(rotation_degrees, crop_rect, image_aspect_ratio);
    let crop_rect_corners = get_crop_rect_corner_positions(crop_rect, image_aspect_ratio);

    let mut bounds = [(-f32::INFINITY, f32::INFINITY); 4];
    let ray_dirs = [
        vec2((-1.0, 0.0)),
        vec2((1.0, 0.0)),
        vec2((0.0, -1.0)),
        vec2((0.0, 1.0)),
    ];
    for i in 0..4 {
        for corner in crop_rect_corners.iter() {
            bounds[i] = update_translation_bounds(
                full_image_edge_segments,
                *corner,
                ray_dirs[i],
                bounds[i],
            );
        }
    }
    bounds
}

pub fn get_crop_rect_upscale_bounds(
    rotation_degrees: f32,
    crop_rect: Rectangle,
    upscale_dir: Vec2<f32>,
    image_aspect_ratio: f32,
) -> (f32, f32) {
    let full_image_edge_segments =
        get_full_image_edge_segments(rotation_degrees, crop_rect, image_aspect_ratio);

    let mut crop_rect_center = crop_rect.center;
    crop_rect_center.x /= image_aspect_ratio;

    let mut relevant_corners = Vec::new();

    if upscale_dir.x == 0.0 && upscale_dir.y == 1.0 {
        relevant_corners = vec![
            crop_rect.max(),
            vec2((crop_rect.min().x, crop_rect.max().y)),
        ]
    } else if upscale_dir.x == 0.0 && upscale_dir.y == -1.0 {
        relevant_corners = vec![
            crop_rect.min(),
            vec2((crop_rect.max().x, crop_rect.min().y)),
        ]
    } else if upscale_dir.x == 1.0 && upscale_dir.y == 0.0 {
        relevant_corners = vec![
            crop_rect.max(),
            vec2((crop_rect.max().x, crop_rect.min().y)),
        ]
    } else if upscale_dir.x == -1.0 && upscale_dir.y == 0.0 {
        relevant_corners = vec![
            crop_rect.min(),
            vec2((crop_rect.min().x, crop_rect.max().y)),
        ]
    } else {
        panic!("unexpected upscale_dir");
    }

    for corner in relevant_corners.iter_mut() {
        corner.x /= image_aspect_ratio;
        *corner = *corner - crop_rect_center;
    }

    let mut bounds = (-f32::INFINITY, f32::INFINITY);

    for corner in relevant_corners.iter() {
        bounds = update_translation_bounds(full_image_edge_segments, *corner, upscale_dir, bounds);
    }

    bounds
}


pub fn handle_new_crop_rect(session: &mut Session, mut transient_edit: Edit, new_crop_rect: Rectangle) {
    if transient_edit.crop_rect != Some(new_crop_rect) {
        if transient_edit.crop_rect.is_none() && new_crop_rect == Rectangle::regular() {
            return;
        }
        let old_crop_rect = transient_edit
            .crop_rect
            .clone()
            .unwrap_or(Rectangle::regular());
        let transform_xy = |x: &mut f32, y: &mut f32| {
            let abs_xy =
                old_crop_rect.min() + (old_crop_rect.max() - old_crop_rect.min()) * vec2((*x, *y));
            let xy = (abs_xy - new_crop_rect.min()) / (new_crop_rect.max() - new_crop_rect.min());
            *x = xy.x;
            *y = xy.y;
        };
        let transform_xy_size = |x_size: &mut f32, y_size: &mut f32| {
            let size = vec2((*x_size, *y_size)) * (old_crop_rect.max() - old_crop_rect.min())
                / (new_crop_rect.max() - new_crop_rect.min());
            *x_size = size.x;
            *y_size = size.y;
        };
        for masked_edit in transient_edit.masked_edits.iter_mut() {
            for term in masked_edit.mask.terms.iter_mut() {
                let prim = &mut term.primitive;
                match prim {
                    MaskPrimitive::RadialGradient(ref mut m) => {
                        transform_xy(&mut m.center_x, &mut m.center_y);
                        transform_xy_size(&mut m.radius_x, &mut m.radius_y);
                    }
                    MaskPrimitive::LinearGradient(ref mut m) => {
                        transform_xy(&mut m.begin_x, &mut m.begin_y);
                        transform_xy(&mut m.saturate_x, &mut m.saturate_y);
                    }
                    MaskPrimitive::Global(_) => {}
                }
            }
        }
        transient_edit.crop_rect = Some(new_crop_rect);
        session.editor.update_transient_edit(transient_edit, false);
    }
}