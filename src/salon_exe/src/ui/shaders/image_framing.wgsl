struct VertexOut {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Params {
    image_color_space: u32,

    frame_aspect_ratio: f32,
    frame_gap: f32,
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

    out.position = vec4(coords, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let uv_in_output = in.uv;

    let image_size = textureDimensions(tex);
    let image_aspect_ratio = f32(image_size.x) / f32(image_size.y);
    var output_size = vec2<u32>(0u, 0u);
    if params.frame_aspect_ratio >= image_aspect_ratio {
        output_size.y = u32((1.0 + params.frame_gap) * f32(image_size.y));
        output_size.x = u32(f32(output_size.y) * params.frame_aspect_ratio);
    }
    else {
        output_size.x = u32((1.0 + params.frame_gap) * f32(image_size.x));
        output_size.y = u32(f32(output_size.x) / params.frame_aspect_ratio);
    }
    
    let pos_in_output = vec2<u32>(uv_in_output * vec2<f32>(output_size - 1u));

    let offset = (output_size - image_size) / 2;
    let pos_in_input = pos_in_output - offset;
    var color = vec4(1.0);
    
    let uv_in_input = vec2<f32>(pos_in_input) / vec2<f32>(image_size - 1u);
    let color_in_input = textureSample(tex, tex_sampler, uv_in_input);
    
    if (uv_in_input.x >= 0.0 && uv_in_input.y >= 0.0 && uv_in_input.x < 1.0 && uv_in_input.y < 1.0){
        color = color_in_input;
        if (params.image_color_space == COLOR_SPACE_LINEAR_RGB) {
            color = vec4(linear_to_srgb(color.rgb), 1.0);
        }
    } 

    return color;
}
