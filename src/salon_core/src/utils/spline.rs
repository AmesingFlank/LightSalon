use super::vec::{dot_vec4, vec2, vec4, Vec2};

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
    let ts = vec4((t3, t2, t, 1.0));
    let b_minus_1 = dot_vec4(ts, vec4((-1.0, 2.0, -1.0, 0.0)));
    let b_0 = dot_vec4(ts, vec4((3.0, -5.0, 0.0, 2.0)));
    let b_1 = dot_vec4(ts, vec4((-3.0, 4.0, 1.0, 0.0)));
    let b_2 = dot_vec4(ts, vec4((1.0, -1.0, 0.0, 0.0)));

    return (p_minus_1 * b_minus_1 + p_0 * b_0 + p_1 * b_1 + p_2 * b_2) * 0.5;
}

pub struct EvaluatedSpline {
    pub y_vals: Vec<f32>,
}

impl EvaluatedSpline {
    pub fn from_control_points(control_points: &Vec<(f32, f32)>) -> EvaluatedSpline {
        let n = control_points.len();
        let p_minus_1 =
            vec2(control_points[0]) + (vec2(control_points[0]) - vec2(control_points[1]));
        let p_N = vec2(control_points[n - 1])
            + (vec2(control_points[n - 1]) - vec2(control_points[n - 2]));

        let mut interpolated_points = Vec::with_capacity(n + 2);
        interpolated_points.push(p_minus_1);
        for p in control_points.iter() {
            interpolated_points.push(vec2(*p));
        }
        interpolated_points.push(p_N);

        let mut curr_p0 = 1usize;
        let mut curr_p0_curve_points = Vec::with_capacity(1001);

        let mut result = EvaluatedSpline { y_vals: Vec::new() };
        for i in 0..=1000 {
            let x = i as f32 / 1000.0;
            let mut y = 0.0;
            if x <= control_points[0].0 {
                y = control_points[0].1;
            } else if x >= control_points[n - 1].0 {
                y = control_points[n - 1].1;
            } else {
                while !(interpolated_points[curr_p0].x <= x
                    && x <= interpolated_points[curr_p0 + 1].x)
                {
                    curr_p0 = curr_p0 + 1;
                    curr_p0_curve_points.clear();
                }
                let p_minus_1 = interpolated_points[curr_p0 - 1];
                let p_0 = interpolated_points[curr_p0];
                let p_1 = interpolated_points[curr_p0 + 1];
                let p_2 = interpolated_points[curr_p0 + 2];
                let num_steps = (p_1.x - p_0.x) / 0.0005;
                let step = 1.0 / num_steps as f32;
                if curr_p0_curve_points.len() < 2 {
                    curr_p0_curve_points.push(catmull_rom_spline(p_minus_1, p_0, p_1, p_2, 0.0));
                    curr_p0_curve_points.push(catmull_rom_spline(p_minus_1, p_0, p_1, p_2, step));
                }
                while x >= curr_p0_curve_points.last().unwrap().x {
                    let next_t = curr_p0_curve_points.len() as f32 * step;
                    curr_p0_curve_points.push(catmull_rom_spline(p_minus_1, p_0, p_1, p_2, next_t));
                }
                let q0 = curr_p0_curve_points[curr_p0_curve_points.len()-2];
                let q1 = curr_p0_curve_points[curr_p0_curve_points.len()-1];

                let s = (x-q0.x) / (q1.x - q0.x);
                y = q0.y + (q1.y - q0.y) * s;
            }
            y = y.max(0.0).min(1.0);
            result.y_vals.push(y);
        }
        result
    }
}
