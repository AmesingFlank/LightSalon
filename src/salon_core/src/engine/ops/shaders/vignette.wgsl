

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
    //var Luv = XYZ_to_Luv(rgb_to_XYZ(rgb));

    let uv = vec2(f32(global_id.x) / f32(input_size.x), f32(global_id.y) / f32(input_size.y));

    let inner = 0.6;
    let outer = 1.2;
    let strength = -params.value * 0.01;

    let edge = length(abs(uv * 2.0 - 1.0));
    let vignette = 1.0 - strength * smoothstep(inner, outer, edge);

    rgb *= vignette;

    textureStore(output, global_id.xy, vec4<f32>(rgb, 1.0));
}