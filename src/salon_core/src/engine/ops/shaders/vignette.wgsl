

@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var output: texture_storage_2d<rgba16float, write>;

struct Params {
    value: f32,
};

@group(0) @binding(2)
var<uniform> params: Params;

@compute
@workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let input_size = textureDimensions(input);
    if(global_id.x >= input_size.x || global_id.y >= input_size.y){
        return;
    }
    var rgb = textureLoad(input, global_id.xy, 0).rgb;
    var hsl = rgb_to_hsl(rgb);

    let max_dim = f32(max(input_size.x, input_size.y));
    let radius = max_dim * 0.5;
    let dist_from_center = abs(vec2<f32>(global_id.xy) - vec2<f32>(input_size) * 0.5);

    let coeff = params.value * 0.02;
    hsl.z *= exp(coeff * dist_from_center.x / radius) * exp(coeff * dist_from_center.y / radius);

    rgb = hsl_to_rgb(hsl);
    textureStore(output, global_id.xy, vec4<f32>(rgb, 1.0));
}