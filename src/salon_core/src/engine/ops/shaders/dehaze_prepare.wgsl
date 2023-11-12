const half_group_size: u32 = 4u;
const group_size: u32 = 8u;
const twice_group_size: u32 = 16u;

@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var output: texture_storage_2d<rgba16float, write>;

struct LocalPatch {
    dark_channels: array<array<f32, twice_group_size>, twice_group_size>,
};

var<workgroup> local_patch: LocalPatch;

const num_bins: i32 = 255;

struct Airlight {
    estimated_airlight: f32,
}

@group(0) @binding(2)
var<storage, read_write> airlight: Airlight; 

fn is_in_image(xy: vec2<u32>, input_size: vec2<u32>) -> bool {
    if (xy.x >= 0u && xy.y >= 0u && xy.x < input_size.x && xy.y < input_size.y) {
        return true;
    }
    return false;
}

fn get_dark_channel(xy: vec2<u32>, input_size: vec2<u32>) -> f32 {
    var result = 1.0;
    if (is_in_image(xy, input_size)) {
        var pixel = textureLoad(input, xy, 0).rgb;
        var dark_channel = min(pixel.r, min(pixel.g, pixel.b));
        //dark_channel = linear_to_srgb_channel(dark_channel);
        result = dark_channel;
    }
    return result;
}

fn f(v: f32) -> f32 {
    let sigma_s = f32(group_size);
    return exp(-v*v) / (2.0 * sigma_s * sigma_s);
}

fn g(v: f32) -> f32 {
    let sigma_r = 0.1;
    return exp(-v*v) / (2.0 * sigma_r * sigma_r);
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

    let my_local_coord = vec2<i32>(local_id.xy) + i32(half_group_size);
    let my_dark_channel = local_patch.dark_channels[my_local_coord.x][my_local_coord.y];

    var sum_num: f32 = 0.0;
    var sum_denom: f32 = 0.0;

    for (var x: i32 = -i32(half_group_size); x <= i32(half_group_size); x += 1) {
        for (var y: i32 = -i32(half_group_size); y <= i32(half_group_size); y += 1) {
            let coord = my_local_coord + vec2<i32>(x, y);
            let global_coord = vec2<i32>(global_id.xy) + vec2(x, y);
            if (is_in_image(vec2<u32>(global_coord), input_size)) {
                let distance = length(vec2(f32(x), f32(y)));
                let dark_channel = local_patch.dark_channels[coord.x][coord.y];
                let value_dist = f32(dark_channel - my_dark_channel);

                let f_dist = f(distance);
                let g_value_dist = g(value_dist);

                sum_num += f_dist * g_value_dist * dark_channel;
                sum_denom += f_dist * g_value_dist;
            }
        }
    }
 
    let veil = sum_num / sum_denom;

    let A = 1.0;

    var t = 1.0 - 0.95 * veil / A;
    t = max(t, 0.1);

    var c = textureLoad(input, global_id.xy, 0).rgb;
    
    let dehazed = (c - A) / t + A; 

    textureStore(output, global_id.xy, vec4<f32>(dehazed, 1.0));
}