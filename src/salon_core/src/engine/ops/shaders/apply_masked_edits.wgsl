

@group(0) @binding(0)
var original: texture_2d<f32>;

@group(0) @binding(1)
var edited: texture_2d<f32>;

@group(0) @binding(2)
var mask: texture_2d<f32>;

@group(0) @binding(3)
var output: texture_storage_2d<rgba16float, write>;


@compute
@workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let output_size = textureDimensions(output);
    if(global_id.x >= output_size.x || global_id.y >= output_size.y){
        return;
    }
    let c0 = textureLoad(original, global_id.xy, 0).rgb;
    let c1 = textureLoad(edited, global_id.xy, 0).rgb;
    let t = textureLoad(mask, global_id.xy, 0).r;
    let c = mix(c0, c1, t);
    textureStore(output, global_id.xy, vec4(c, 1.0));
}