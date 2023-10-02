

@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var output: texture_storage_2d<rgba8unorm, write>;

struct Params {
    value: f32,
};

@group(0) @binding(2)
var<uniform> params: Params;

@compute
@workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var rgb = textureLoad(input, global_id.xy, 0).rgb;
    rgb += params.value * 0.01;
    rgb = clamp(rgb, vec3(0.0), vec3(1.0));
    textureStore(output, global_id.xy, vec4<f32>(rgb, 1.0));
}