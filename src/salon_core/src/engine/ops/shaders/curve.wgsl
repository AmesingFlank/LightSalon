const NUM_STEPS: u32 = 255u;
const NUM_Y_VALS: u32 = 256u; 

@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var output: texture_storage_2d<rgba16float, write>;

struct Params {
    y_val: array<f32, NUM_Y_VALS>,
    x_max: f32,
};

@group(0) @binding(2)
var<storage, read> params: Params;

@compute
@workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let input_size = textureDimensions(input);
    if(global_id.x >= input_size.x || global_id.y >= input_size.y){
        return;
    }
    var rgb = textureLoad(input, global_id.xy, 0).rgb;

    var srgb = linear_to_srgb(rgb);
    var luma = dot(srgb, vec3(0.2126, 0.7152, 0.0722));

    let index = u32(luma * (params.x_max / f32(NUM_STEPS)));
    var new_luma = params.y_val[index];

    srgb *= new_luma / luma;

    rgb = srgb_to_linear(srgb);
    textureStore(output, global_id.xy, vec4<f32>(srgb, 1.0));
}