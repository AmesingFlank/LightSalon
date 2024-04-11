@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var output: texture_storage_2d<rgba16float, write>;


@compute
@workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let input_size = textureDimensions(input);
    let output_size = textureDimensions(output);
    if(global_id.x >= output_size.x || global_id.y >= output_size.y){
        return;
    }

    let offset = (output_size - input_size) / 2;
    let pos = global_id.xy - offset;
    var color = vec4(1.0);
    if (pos.x >= 0 && pos.y >= 0 && pos.x < input_size.x && pos.y < input_size.y){
        color = textureLoad(input, pos, 0);
    }
    textureStore(output, global_id.xy, color);
}