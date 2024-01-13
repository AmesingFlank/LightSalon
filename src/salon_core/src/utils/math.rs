use num::Num;

use super::vec::{vec3, Vec2, Vec3, Vec4};

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
