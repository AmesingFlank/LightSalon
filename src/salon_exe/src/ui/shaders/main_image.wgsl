struct VertexOut {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Params {
    image_color_space: u32,
    indicate_mask: u32,

    min_u: f32,
    min_v: f32,
    max_u: f32,
    max_v: f32,
};

@group(0) @binding(0)
var<uniform> params: Params;

@group(0) @binding(1)
var tex: texture_2d<f32>;

@group(0)@binding(2)
var tex_sampler: sampler;

@group(0) @binding(3)
var mask: texture_2d<f32>;

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

    out.position = vec4(coords, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let min_uv = vec2(params.min_u, params.min_v);
    let max_uv = vec2(params.max_u, params.max_v);
    var uv = in.uv;
    uv = min_uv + uv * (max_uv - min_uv);
    var color = textureSample(tex, tex_sampler, uv).rgb;

    let image_size = textureDimensions(tex);
    if (params.image_color_space == COLOR_SPACE_LINEAR_RGB) {
        color = linear_to_srgb(color);
    }

    if (params.indicate_mask != 0u) {
        let mask_value = textureSample(mask, tex_sampler, uv).r;
        let mask_color = vec3(1.0, 0.1, 0.1);
        color = mix(color, mask_color, mask_value * 0.5);
    }

    return vec4(color, 1.0);
}
