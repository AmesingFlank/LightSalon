struct Mask {
    center_x: f32,
    center_y: f32,
    radius_x: f32,
    radius_y: f32,
    inner_ellipse_ratio: f32,
    rotation: f32,
};

@group(0) @binding(0)
var<uniform> mask: Mask;

@group(0) @binding(1)
var output: texture_storage_2d<rgba8unorm, write>; 

@compute
@workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let output_size = textureDimensions(output);
    if(global_id.x >= output_size.x || global_id.y >= output_size.y){
        return;
    }

    let x = f32(global_id.x) / f32(output_size.x - 1u);
    let y = f32(global_id.y) / f32(output_size.y - 1u);

    textureStore(output, global_id.xy, vec4<f32>(1.0));
}