const max_def_points:u32 = 16u;
const max_result_points:u32 = 256u;

struct CurveDef {
    points: array<vec4<f32>, max_def_points>,
    num_points: u32,
};

struct CurvePoints {
    points: array<vec4<f32>, max_result_points>,
    num_points: u32,
};