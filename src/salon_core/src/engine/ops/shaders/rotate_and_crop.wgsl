@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var tex_sampler: sampler;

@group(0) @binding(2)
var output: texture_storage_2d<rgba16float, write>;

struct Params {
    center_x: f32,
    center_y: f32,
    rotation_radians: f32,
};

@group(0) @binding(3)
var<uniform> params: Params;

@compute
@workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let input_size = textureDimensions(input);
    let output_size = textureDimensions(output);
    if(global_id.x >= output_size.x || global_id.y >= output_size.y){
        return;
    }

    let size = vec2<f32>(output_size) / vec2<f32>(input_size);

    var uv = vec2<f32>(global_id.xy) / vec2<f32>(input_size - 1u);

    let input_image_aspect_ratio = f32(input_size.y) / f32(input_size.x);

    // centered around (0, 0)
    uv = uv - 0.5 * size;

    uv.x /= input_image_aspect_ratio;

    let rotation_col0 = vec2 (
        cos(-params.rotation_radians),
        sin(-params.rotation_radians)
    );
    let rotation_col1 = vec2 (
        -sin(-params.rotation_radians),
        cos(-params.rotation_radians)
    );
    let rotation = mat2x2(rotation_col0, rotation_col1); 

    uv = rotation * uv;

    uv.x *= input_image_aspect_ratio;

    uv = uv + vec2(params.center_x, params.center_y);

    var c = textureSampleLevel(input, tex_sampler, uv, 0.0).rgb;
    textureStore(output, global_id.xy, vec4<f32>(c, 1.0));
}