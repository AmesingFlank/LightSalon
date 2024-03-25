struct VertexOut {
    @location(0) uv: vec2<f32>,
    @location(1) position_interpolated: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Params {
    image_color_space: u32,

    rotation_radians: f32,

    center_x: f32,
    center_y: f32,
    width: f32,
    height: f32,
    crop_rect_width: f32,
    crop_rect_height: f32,
    render_target_aspect_ratio: f32,
};

@group(0) @binding(0)
var<uniform> params: Params;

@group(0) @binding(1)
var tex: texture_2d<f32>;

@group(0)@binding(2)
var tex_sampler: sampler;

var<private> vertex_corner_coords: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, -1.0),

    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0, 1.0),
);

@vertex
fn vs_main(@builtin(vertex_index) vertex_idx: u32) -> VertexOut {
    var out: VertexOut;

    var coords = vertex_corner_coords[vertex_idx];

    out.uv = (coords * vec2(1.0, -1.0) + 1.0) * 0.5;

    var pos = vec2(params.center_x, params.center_y) + coords * vec2(params.width, params.height) * 0.5;

    out.position = vec4(pos, 0.0, 1.0);
    out.position_interpolated = out.position;
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

    let frag_pos = in.position_interpolated;
    if (abs(frag_pos.x) > params.crop_rect_width * 0.5 || abs(frag_pos.y) > params.crop_rect_height * 0.5) {
        color *= 0.3;
    }

    return vec4(color, 1.0);
}
