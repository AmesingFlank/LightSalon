const half_group_size: u32 = 4u;
const group_size: u32 = 8u;
const twice_group_size: u32 = 16u;

@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var output: texture_storage_2d<rgba16float, write>;

struct Params {
    value: f32,
};

@group(0) @binding(2)
var<uniform> params: Params;

struct LocalPatch {
    dark_channels: array<array<u32, twice_group_size>, twice_group_size>,
    sum_variance: atomic<u32>,
};

var<workgroup> local_patch: LocalPatch;

fn is_in_image(xy: vec2<u32>, input_size: vec2<u32>) -> bool {
    if (xy.x >= 0u && xy.y >= 0u && xy.x < input_size.x && xy.y < input_size.y) {
        return true;
    }
    return false;
}

fn get_dark_channel(xy: vec2<u32>, input_size: vec2<u32>) -> u32 {
    var result = 255u;
    if (is_in_image(xy, input_size)) {
        var pixel = textureLoad(input, xy, 0).rgb;
        var dark_channel = min(pixel.r, min(pixel.g, pixel.b));
        //dark_channel = linear_to_srgb_channel(dark_channel);
        result = u32(dark_channel * 255.0);
    }
    return result;
}

@compute
@workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(local_invocation_id) local_id: vec3<u32>) {
    let input_size = textureDimensions(input);
    if(global_id.x >= input_size.x || global_id.y >= input_size.y){
        return;
    }

    var quadrant: vec2<i32> = vec2(1, 1);
    if (local_id.x < half_group_size) {
        quadrant.x = -1;
    }
    if (local_id.y < half_group_size) {
        quadrant.y = -1;
    }

    local_patch.sum_variance = 0u;

    for (var x: i32 = 0; x < 2; x += 1) {
        for (var y: i32 = 0; y < 2; y += 1) {
            let delta = vec2<i32>(x, y) * quadrant * i32(half_group_size);
            let global_coord = vec2<i32>(global_id.xy) + delta;
            let local_coord = vec2<i32>(local_id.xy) + delta + i32(half_group_size);
            if (is_in_image(vec2<u32>(global_coord), input_size)) {
                let dark = get_dark_channel(vec2<u32>(global_coord), input_size);
                local_patch.dark_channels[local_coord.x][local_coord.y] = dark;
            }
        }
    }

    workgroupBarrier(); 

    let local_coord = vec2<i32>(local_id.xy) + i32(half_group_size);
    var sum: u32 = 0u;
    var sum_squares: u32 = 0u;
    var count: u32 = 0u;
    var max_dark = 0u;

    for (var x: i32 = -i32(half_group_size); x <= i32(half_group_size); x += 1) {
        for (var y: i32 = -i32(half_group_size); y <= i32(half_group_size); y += 1) {
            let coord = local_coord + vec2<i32>(x, y);
            let global_coord = vec2<i32>(global_id.xy) + vec2(x, y);
            if (is_in_image(vec2<u32>(global_coord), input_size)) {
                let dark = local_patch.dark_channels[coord.x][coord.y];
                sum += dark;
                sum_squares += dark * dark;
                count += 1u;
                max_dark = max(max_dark, dark);
            }
        }
    }

    let local_mean = sum / count;
    let local_variance = sum_squares / count - local_mean * local_mean;
    atomicAdd(&local_patch.sum_variance, local_variance);

    workgroupBarrier(); 

    let noise = local_patch.sum_variance / (group_size * group_size);

    let mean_f = f32(local_mean) / 255.0;
    let var_f = f32(local_variance) / (255.0 * 255.0);
    let noise_f = f32(noise) / (255.0 * 255.0);

    let this_pixel_dark = f32(get_dark_channel(global_id.xy, input_size)) / 255.0;

    let veil = mean_f + max(var_f - noise_f, 0.0) / var_f * (this_pixel_dark - mean_f);


    let A = 0.5;

    var t = 1.0 - 0.95 * veil / A;
    t = max(t, 0.1);

    var c = textureLoad(input, global_id.xy, 0).rgb;

    //c = linear_to_srgb(c);
    
    let dehazed = (c - A) / t + A;

    let coeff = params.value * 0.01;
    c = c * (1.0 - coeff) + dehazed * coeff;

    //c = vec3(t);

    //c = srgb_to_linear(c);

    textureStore(output, global_id.xy, vec4<f32>(c, 1.0));
}