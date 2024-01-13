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

pub fn dot_vec2(a: Vec2<f32>, b: Vec2<f32>) -> f32 {
    a.x * b.x + a.y * b.y
}

pub fn dot_vec3(a: Vec3<f32>, b: Vec3<f32>) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

pub fn dot_vec4(a: Vec4<f32>, b: Vec4<f32>) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w
}

pub fn clamp_vec3(v: Vec3<f32>, min: Vec3<f32>, max: Vec3<f32>) -> Vec3<f32> {
    vec3((
        clamp(v.x, min.x, max.x),
        clamp(v.y, min.y, max.y),
        clamp(v.z, min.z, max.z),
    ))
}

pub fn length_vec2(v: Vec2<f32>) -> f32 {
    dot_vec2(v, v).sqrt()
}

pub fn length_vec3(v: Vec3<f32>) -> f32 {
    dot_vec3(v, v).sqrt()
}

pub fn length_vec4(v: Vec4<f32>) -> f32 {
    dot_vec4(v, v).sqrt()
}
