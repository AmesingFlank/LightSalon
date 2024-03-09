@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var tex_sampler: sampler;

@group(0) @binding(2)
var output: texture_storage_2d<rgba16float, write>;

struct Params {
    center_x: f32,
    center_y: f32,
    size_x: f32,
    size_y: f32,
};

@group(0) @binding(3)
var<uniform> params: Params;

@compute
@workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let input_size = textureDimensions(input);
    let output_size = textureDimensions(output);
    if(global_id.x >= output_size.x || global_id.y >= output_size.y){
        return;
    }

    let x = params.center_x - 0.5 * params.size_x + f32(global_id.x) / f32(input_size.x - 1u);
    let y = params.center_y - 0.5 * params.size_y + f32(global_id.y) / f32(input_size.y - 1u);
    
    var c = textureSampleLevel(input, tex_sampler, vec2(x, y), 0.0).rgb;
    textureStore(output, global_id.xy, vec4<f32>(c, 1.0));
}