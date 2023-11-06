use std::cmp::{max, min};

use num::complex::ComplexFloat;

use super::{
    math::{clamp_vec3, mix, step},
    vec::{vec3, vec4, Vec3, Vec4},
};

// hmmc: hue, channel-wise min, Max and chroma (i.e. max-min)
fn rgb_to_hmmc(rgb: Vec3<f32>) -> Vec4<f32> {
    let M = rgb.x.max(rgb.y).max(rgb.z);
    let m = rgb.x.min(rgb.y).min(rgb.z);
    let chroma = M - m;
    let dc = vec3((rgb.y - rgb.z, rgb.z - rgb.x, rgb.x - rgb.y)) / chroma.max(0.001);
    let mut hue = dc.z + 4.0;
    hue = mix(hue, dc.y + 2.0, step(M, rgb.y));
    hue = mix(hue, dc.x, step(M, rgb.x));
    hue = hue / 6.0;
    if hue < 0.0 {
        hue = hue + 1.0;
    }
    return vec4((hue, m, M, chroma));
}

fn hue_to_rgb(hue: f32) -> Vec3<f32> {
    let r = (hue * 6.0 - 3.0).abs() - 1.0;
    let g = -(hue * 6.0 - 2.0).abs() + 2.0;
    let b = -(hue * 6.0 - 4.0).abs() + 2.0;
    return clamp_vec3(
        vec3((r, g, b)),
        vec3((0.0, 0.0, 0.0)),
        vec3((1.0, 1.0, 1.0)),
    );
}

pub fn rgb_to_hsl(rgb: Vec3<f32>) -> Vec3<f32> {
    let hmmc = rgb_to_hmmc(rgb);
    let sum = hmmc.y + hmmc.z;
    let den = 1.0 - (sum - 1.0).abs();
    return vec3((hmmc.x, hmmc.w / den.max(0.001), sum * 0.5));
}

pub fn hsl_to_rgb(hsl: Vec3<f32>) -> Vec3<f32> {
    let rgb = hue_to_rgb(hsl.x);
    let chroma = (1.0 - (2.0 * hsl.z - 1.0).abs()) * hsl.y;
    return (rgb - vec3((0.5, 0.5, 0.5))) * chroma + vec3((hsl.z, hsl.z, hsl.z));
}
