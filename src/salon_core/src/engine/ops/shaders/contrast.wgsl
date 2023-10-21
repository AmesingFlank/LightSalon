

@group(0) @binding(0)
var input: texture_2d<f32>;

struct BasicStats {
    mean_rgba: vec4<f32>,
};

@group(0) @binding(1)
var<storage, read_write> basic_stats: BasicStats;

@group(0) @binding(2)
var output: texture_storage_2d<rgba16float, write>;

struct Params {
    contrast: f32,
};

@group(0) @binding(3)
var<uniform> params: Params;

@compute
@workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let input_size = textureDimensions(input);
    if(global_id.x >= input_size.x || global_id.y >= input_size.y){
        return;
    }

    let mean_rgb = basic_stats.mean_rgba.rgb;

    var rgb = textureLoad(input, global_id.xy, 0).rgb;

    var diff = rgb - mean_rgb;
    diff = diff * (1.0 + params.contrast * 0.01 * 0.5);

    rgb = mean_rgb + diff;

    textureStore(output, global_id.xy, vec4<f32>(rgb, 1.0));
}