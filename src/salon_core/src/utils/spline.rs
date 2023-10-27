use super::vec::{Vec2, vec4, dot_vec4};

// https://github.com/AmesingFlank/OxfordCSNotes/blob/master/GMOD18-19/Lecture9_GMod%20Drawing%20splines%3B%20degree%20elevation%2C%20sculptured%20surface%20patches.pdf
fn catmull_rom_spline(
    p_minus_1: Vec2<f32>,
    p_0: Vec2<f32>,
    p_1: Vec2<f32>,
    p_2: Vec2<f32>,
    t: f32,
) -> Vec2<f32> {
    let t2 = t * t;
    let t3 = t2 * t;
    let ts = vec4(t3, t3, t2, 1.0);
    let b_minus_1 = dot_vec4(ts, vec4(-1.0, 2.0, -1.0, 0.0));
    let b_0 = dot_vec4(ts, vec4(3.0, -5.0, 0.0, 2.0));
    let b_1 = dot_vec4(ts, vec4(-3.0, 4.0, 1.0, 0.0));
    let b_2 = dot_vec4(ts, vec4(1.0, -1.0, 0.0, 0.0));

    return (p_minus_1 * b_minus_1 + p_0 * b_0 + p_1 * b_1 + p_2 * b_2) * 0.5;
}
