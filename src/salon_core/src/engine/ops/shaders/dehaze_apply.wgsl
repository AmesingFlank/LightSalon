

@group(0) @binding(0)
var input: texture_2d<f32>;


@group(0) @binding(1)
var dehazed: texture_2d<f32>;

@group(0) @binding(2)
var output: texture_storage_2d<rgba16float, write>;

struct Params {
    value: f32,
};

@group(0) @binding(3)
var<uniform> params: Params;

@compute
@workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let input_size = textureDimensions(input);
    if(global_id.x >= input_size.x || global_id.y >= input_size.y){
        return;
    }
    let original_color = textureLoad(input, global_id.xy, 0).rgb;
    let dehazed_color = textureLoad(dehazed, global_id.xy, 0).rgb;
    let t = params.value * 0.01;
    let result = mix(original_color, dehazed_color, t);
    textureStore(output, global_id.xy, vec4<f32>(result, 1.0));
}