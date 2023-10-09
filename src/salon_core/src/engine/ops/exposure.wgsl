

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
@workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var c = textureLoad(input, global_id.xy, 0).rgb;
    c = c * pow(2.0, params.value / 2.2);
    textureStore(output, global_id.xy, vec4<f32>(c, 1.0));
}