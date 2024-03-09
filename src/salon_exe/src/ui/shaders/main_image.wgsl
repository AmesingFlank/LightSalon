struct VertexOut {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Params {
    image_color_space: u32,

    indicate_mask: u32,

    crop_center_x: f32,
    crop_center_y: f32,
    crop_size_x: f32,
    crop_size_y: f32,
};

@group(0) @binding(0)
var<uniform> params: Params;

@group(0) @binding(1)
var tex: texture_2d<f32>;

@group(0)@binding(2)
var tex_sampler: sampler;

@group(0) @binding(3)
var mask: texture_2d<f32>;

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

    let crop_min_x = params.crop_center_x - 0.5 *  params.crop_size_x;
    let crop_min_y = params.crop_center_y - 0.5 *  params.crop_size_y;
    let crop_max_x = params.crop_center_x + 0.5 *  params.crop_size_x;
    let crop_max_y = params.crop_center_y + 0.5 *  params.crop_size_y;
    if (uv.x < crop_min_x  || uv.x > crop_max_x || uv.y < crop_min_y || uv.y > crop_max_y){
        color *= 0.3;
    }

    if (params.indicate_mask != 0u) {
        let mask_value = textureSample(mask, tex_sampler, uv).r;
        let mask_color = vec3(1.0, 0.1, 0.1);
        color = mix(color, mask_color, mask_value * 0.5);
    }

    return vec4(color, 1.0);
}
