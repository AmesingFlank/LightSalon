

@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var output: texture_storage_2d<rgba16float, write>;

struct Params {
    src_color_space: u32,
    dest_color_space: u32,
};

@group(0) @binding(2)
var<uniform> params: Params;

@compute
@workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var c = textureLoad(input, global_id.xy, 0).rgb;
    if (params.src_color_space == COLOR_SPACE_LINEAR_RGB) {
        if (params.dest_color_space == COLOR_SPACE_sRGB) {
            c = linear_to_srgb(c); 
        }
    }
    else if (params.src_color_space == COLOR_SPACE_sRGB) {
        if (params.dest_color_space == COLOR_SPACE_LINEAR_RGB) {
            c = srgb_to_linear(c); 
        }
    }
    textureStore(output, global_id.xy, vec4<f32>(c, 1.0));
}