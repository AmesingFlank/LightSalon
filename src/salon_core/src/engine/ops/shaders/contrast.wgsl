

@group(0) @binding(0)
var input: texture_2d<f32>;

struct BasicStats {
    meana: vec4<f32>,
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

fn opposite_from_mean(x: f32, mean_x: f32) -> f32 {
    if (x < mean_x) {
        return 0.0;
    }
    else if (x > mean_x) {
        return max(1.0, x+0.01); // need to consider HDR images
    }
    else {
        // x == mean_x
        return x;
    }
}

@compute
@workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let input_size = textureDimensions(input);
    if(global_id.x >= input_size.x || global_id.y >= input_size.y){
        return;
    }

    let mean = basic_stats.meana.rgb;
    var rgb = textureLoad(input, global_id.xy, 0).rgb;
    let opposite = vec3(
        opposite_from_mean(rgb.r, mean.r),
        opposite_from_mean(rgb.g, mean.g),
        opposite_from_mean(rgb.b, mean.b),
    );

    var diff = rgb - mean;

    var speed = (rgb - mean) / (opposite - mean);

    speed = min(speed, (opposite - rgb) / (opposite - mean));

    diff = diff * (1.0 + params.contrast * 0.01 * speed) ;
    rgb = mean + diff;

    textureStore(output, global_id.xy, vec4<f32>(rgb, 1.0));
}