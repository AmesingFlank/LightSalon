

@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0)@binding(1)
var tex_sampler: sampler;

@group(0) @binding(2)
var output: texture_storage_2d<rgba16float, write>;

struct Params {
    contrast: f32,
    max_lod: f32,
};

@group(0) @binding(3)
var<uniform> params: Params;

@compute
@workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let input_size = textureDimensions(input);
    if(global_id.x >= input_size.x || global_id.y >= input_size.y){
        return;
    }

    let uv = vec2(f32(global_id.x) / f32(input_size.x), f32(global_id.y) / f32(input_size.y));

    let selected_lod = abs(params.contrast) * params.max_lod * 0.5;

    let mean_rgb = textureSampleLevel(input, tex_sampler, uv, selected_lod).rgb;
    let mean_hsv = rgb_to_hsv(mean_rgb);

    var rgb = textureLoad(input, global_id.xy, 0).rgb;
    var hsv = rgb_to_hsv(rgb);

    var l_diff = hsv.z - mean_hsv.z;
    l_diff = l_diff * (1.0 + params.contrast * 0.01 * 0.5);

    hsv.z = mean_hsv.z + l_diff;
    rgb = hsv_to_rgb(hsv);

    textureStore(output, global_id.xy, vec4<f32>(rgb, 1.0));
}