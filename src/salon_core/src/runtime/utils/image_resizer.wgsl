@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var tex_sampler: sampler;

@group(0) @binding(2)
var output: texture_storage_2d<IMAGE_FORMAT, write>;

struct Params {
    lod: f32,
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

    let x = f32(global_id.x) / f32(output_size.x - 1u);
    let y = f32(global_id.y) / f32(output_size.y - 1u);
    
    var c = textureSampleLevel(input, tex_sampler, vec2(x, y), params.lod).rgb;
    textureStore(output, global_id.xy, vec4<f32>(c, 1.0));
}