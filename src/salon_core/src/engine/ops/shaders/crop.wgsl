@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var output: texture_storage_2d<rgba16float, write>;

struct Params {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
};

@group(0) @binding(2)
var<uniform> params: Params;

@compute
@workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let input_size = textureDimensions(input);
    let output_size = textureDimensions(output);
    if(global_id.x >= output_size.x || global_id.y >= output_size.y){
        return;
    }
    
    let x_offset = params.min_x * f32(input_size.x);
    let y_offset = params.min_y * f32(input_size.y);

    let source_coords = vec2(u32(x_offset), u32(y_offset)) + global_id.xy;
    var c = textureLoad(input, source_coords, 0).rgb;
    textureStore(output, global_id.xy, vec4<f32>(c, 1.0));
}