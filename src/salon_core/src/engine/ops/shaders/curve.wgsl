const NUM_STEPS: u32 = 255u;
const NUM_Y_VALS: u32 = 256u; 

@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var output: texture_storage_2d<rgba16float, write>;

struct Curve {
    y_val: array<f32, NUM_Y_VALS>,
    x_max: f32,
};

@group(0) @binding(2)
var<storage, read> curve: Curve;

struct Params {
    adjust_r: u32,
    adjust_g: u32,
    adjust_b: u32,
};

@group(0) @binding(3)
var<storage, read> params: Params;

fn apply(f: f32) -> f32 {
    let index = u32(f / (curve.x_max / f32(NUM_STEPS)));
    return curve.y_val[index];
}

@compute
@workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let input_size = textureDimensions(input);
    if(global_id.x >= input_size.x || global_id.y >= input_size.y){
        return;
    }
    var rgb = textureLoad(input, global_id.xy, 0).rgb;

    var srgb = linear_to_srgb(rgb);
    
    if(params.adjust_r != 0u){
        srgb.r = apply(srgb.r);
    }
    if(params.adjust_g != 0u){
        srgb.g = apply(srgb.g);
    }
    if(params.adjust_b != 0u){
        srgb.b = apply(srgb.b);
    }

    rgb = srgb_to_linear(srgb);
    textureStore(output, global_id.xy, vec4<f32>(rgb, 1.0));
}