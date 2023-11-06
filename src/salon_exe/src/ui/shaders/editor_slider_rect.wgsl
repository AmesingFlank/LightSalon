struct VertexOut {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Params {
    color_left: vec4<f32>,
    color_right: vec4<f32>,
    interpolate_in_hsl: f32,
};

@group(0) @binding(0)
var<uniform> params: Params;

var<private> v_positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, -1.0),

    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0, 1.0),
);

@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(v_positions[v_idx], 0.0, 1.0);
    out.uv = (out.position.xy + 1.0) * 0.5;
    out.uv.y = 1.0 - out.uv.y;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let t = 1.0 - in.uv.x;
    if (params.interpolate_in_hsl == 1.0) {
        var hsl_left = rgb_to_hsl(params.color_left.xyz);
        var hsl_right = rgb_to_hsl(params.color_right.xyz);

        if (abs(hsl_right.x - hsl_left.x) > abs(hsl_right.x + 1.0 - hsl_left.x)) {
            hsl_right.x = hsl_right.x + 1.0;
        }
        if (abs(hsl_right.x - hsl_left.x) > abs(hsl_right.x - (hsl_left.x + 1.0))) {
            hsl_left.x = hsl_left.x + 1.0;
        }

        var hsl = t * hsl_left + (1.0 - t) * hsl_right;
        if (hsl.x > 1.0) {
            hsl.x -= 1.0;
        }

        return vec4(hsl_to_rgb(hsl), 1.0);
    }
    else {
        return t * params.color_left + (1.0 - t) * params.color_right;
    }
}
