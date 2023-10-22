

@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var output: texture_storage_2d<rgba16float, write>;

struct Params {
    value: f32,
};

@group(0) @binding(2)
var<uniform> params: Params;

fn influence(l: f32) -> f32 {
    return min((l * l) * (l * l), 1.0);
}

@compute
@workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let input_size = textureDimensions(input);
    if(global_id.x >= input_size.x || global_id.y >= input_size.y){
        return;
    }
    var rgb = textureLoad(input, global_id.xy, 0).rgb;
    var hsv = rgb_to_hsv(rgb);
    hsv.z *= pow(2.0, params.value * 0.01 * influence(hsv.z));
    rgb = hsv_to_rgb(hsv);
    textureStore(output, global_id.xy, vec4<f32>(rgb, 1.0));
}