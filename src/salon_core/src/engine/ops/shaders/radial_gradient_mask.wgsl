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

    var xy = vec2(
        f32(global_id.x) / f32(output_size.x - 1u),
        f32(global_id.y) / f32(output_size.y - 1u)
    );

    xy = xy - vec2(mask.center_x, mask.center_y);

    let rotation_col0 = vec2 (
        cos(mask.rotation),
        sin(mask.rotation)
    );
    let rotation_col1 = vec2 (
        -sin(mask.rotation),
        cos(mask.rotation)
    );
    let rotation = mat2x2(rotation_col0, rotation_col1); 
    xy = rotation * xy;

    let r = (xy.x / mask.radius_x) * (xy.x / mask.radius_x) + (xy.y / mask.radius_y) * (xy.y / mask.radius_y);

    var result = 0.0;

    if (r < mask.inner_ellipse_ratio) {
        result = 1.0;
    }
    else if (r < 1.0) {
        result = (1.0 - r) / (1.0 - mask.inner_ellipse_ratio);
    }

    textureStore(output, global_id.xy, vec4<f32>(result));
}