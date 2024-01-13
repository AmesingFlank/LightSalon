use std::mem::size_of;

use crate::runtime::{Buffer, Runtime};

use super::vec::{vec2, vec4, Vec2, Vec4};

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
    let b_minus_1 = ts.dot(&vec4((-1.0, 2.0, -1.0, 0.0)));
    let b_0 = ts.dot(&vec4((3.0, -5.0, 0.0, 2.0)));
    let b_1 = ts.dot(&vec4((-3.0, 4.0, 1.0, 0.0)));
    let b_2 = ts.dot(&vec4((1.0, -1.0, 0.0, 0.0)));

    return (p_minus_1 * b_minus_1 + p_0 * b_0 + p_1 * b_1 + p_2 * b_2) * 0.5;
}

// Evaluate the y values of a spline where x is in [0, x_max]
pub struct EvaluatedSpline {
    pub y_vals: Vec<f32>,
    pub x_max: f32,
}

impl EvaluatedSpline {
    pub fn from_control_points(
        control_points: &Vec<(f32, f32)>,
        x_max: f32,
        num_steps: u32,
    ) -> EvaluatedSpline {
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
        let mut curr_p0_curve_points = Vec::new();

        let mut result = EvaluatedSpline {
            x_max,
            y_vals: Vec::new(),
        };
        for i in 0..=num_steps {
            let x = i as f32 / num_steps as f32;
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
                let num_t_steps = 2.0 * num_steps as f32 * (p_1.x - p_0.x) / x_max;
                let t_step = 1.0 / num_t_steps;
                if curr_p0_curve_points.len() < 2 {
                    curr_p0_curve_points.push(catmull_rom_spline(p_minus_1, p_0, p_1, p_2, 0.0));
                    curr_p0_curve_points.push(catmull_rom_spline(p_minus_1, p_0, p_1, p_2, t_step));
                }
                while x >= curr_p0_curve_points.last().unwrap().x {
                    let next_t = curr_p0_curve_points.len() as f32 * t_step;
                    curr_p0_curve_points.push(catmull_rom_spline(p_minus_1, p_0, p_1, p_2, next_t));
                }
                let q0 = curr_p0_curve_points[curr_p0_curve_points.len() - 2];
                let q1 = curr_p0_curve_points[curr_p0_curve_points.len() - 1];

                let s = (x - q0.x) / (q1.x - q0.x);
                y = q0.y + (q1.y - q0.y) * s;
            }
            y = y.max(0.0).min(1.0);
            result.y_vals.push(y);
        }
        result
    }

    pub fn write_to_buffer(&self, runtime: &Runtime, buffer: &Buffer) {
        let mut offset = 0;
        runtime.queue.write_buffer(
            &buffer.buffer,
            offset as u64,
            bytemuck::cast_slice(self.y_vals.as_slice()),
        );
        offset += self.y_vals.len() * size_of::<f32>();
        runtime.queue.write_buffer(
            &buffer.buffer,
            offset as u64,
            bytemuck::cast_slice(&[self.x_max]),
        );
    }
}
