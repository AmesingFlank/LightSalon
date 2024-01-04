@group(0) @binding(0)
var mask_0: texture_2d<f32>;

@group(0) @binding(1)
var mask_1: texture_2d<f32>;

@group(0) @binding(2)
var output: texture_storage_2d<rgba8unorm, write>;


@compute
@workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let output_size = textureDimensions(output);
    if(global_id.x >= output_size.x || global_id.y >= output_size.y){
        return;
    }

    let m0 = textureLoad(mask_0, global_id.xy, 0).r;
    let m1 = textureLoad(mask_1, global_id.xy, 0).r;
    let m = max(0.0, m0 - m1);
    textureStore(output, global_id.xy, vec4(vec3(m), 1.0));
}