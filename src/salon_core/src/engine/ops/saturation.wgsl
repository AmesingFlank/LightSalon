

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
    var rgb = textureLoad(input, global_id.xy, 0).rgb;
    var hsl = rgb_to_hsl(rgb);
    hsl.y += params.value * 0.01;
    hsl.y = clamp(hsl.y, 0.0, 1.0);
    rgb = hsl_to_rgb(hsl);
    textureStore(output, global_id.xy, vec4(rgb, 1.0));
}