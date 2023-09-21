

@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var output: texture_storage_2d<rgba8unorm, write>;

@compute
@workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let c = textureLoad(input, global_id.xy, 0);
    textureStore(output, global_id.xy, c);
}