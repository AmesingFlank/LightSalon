

@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var<storage, read_write> output: array<u32>;

struct Params {
    width: u32,
    height: u32,
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
    let c = textureLoad(input, global_id.xy, 0).rgb;
    let r = u32(c.r * 255.0);
    let g = u32(c.g * 255.0);
    let b = u32(c.b * 255.0);
    let a = 255u;

    let pixel = (r) | (g << 8) | (b << 16) | (a << 24);

    let index = global_id.y * params.width + global_id.x;
    output[index] = pixel;
}