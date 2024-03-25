use num::Num;

use super::{
    mat::Mat2x2,
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
