struct VertexOut {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Params {
    image_color_space: u32,

    crop_min_x: f32,
    crop_min_y: f32,
    crop_max_x: f32,
    crop_max_y: f32,
};

@group(0) @binding(0)
var<uniform> params: Params;

@group(0) @binding(1)
var tex: texture_2d<f32>;

@group(0)@binding(2)
var tex_sampler: sampler;

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
    let uv = in.uv;
    var color = textureSample(tex, tex_sampler, uv).rgb;

    let image_size = textureDimensions(tex);
    if (params.image_color_space == COLOR_SPACE_LINEAR_RGB) {
        color = linear_to_srgb(color);
    }

    if (uv.x < params.crop_min_x || uv.x > params.crop_max_x || uv.y < params.crop_min_y || uv.y > params.crop_max_y){
        color *= 0.3;
    }

    return vec4(color, 1.0);
}
