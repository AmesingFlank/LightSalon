struct Mask {
    begin_x: f32,
    begin_y: f32,
    saturate_x: f32,
    saturate_y: f32
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

    let scale = vec2(f32(output_size.x - 1u), f32(output_size.y - 1u));
    let xy = vec2(f32(global_id.x), f32(global_id.y)) / scale;

    let begin = vec2(mask.begin_x, mask.begin_y);
    let saturated = vec2(mask.saturate_x, mask.saturate_y);

    var normal = begin - saturated;
    let transition_length = length(normal);
    normal = normal / transition_length;

    let v = xy - saturated;
    let projected_distance = dot(v, normal) / transition_length;

    var result = 0.0;
    if (projected_distance <= 0.0) {
        result = 1.0;
    }
    else if (projected_distance <= 1.0) {
        result = 1.0 - projected_distance;
    }  

    textureStore(output, global_id.xy, vec4<f32>(result));
}