

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
    min_dark_channels: array<array<atomic<u32>, 4>, 4>,
};

var<workgroup> local_patch: LocalPatch;

fn get_dark_channel(xy: vec2<u32>, input_size: vec2<u32>) -> u32 {
    var result = 255u;
    if (xy.x >= 0u && xy.y >= 0u && xy.x < input_size.x && xy.y < input_size.y) {
        var pixel = textureLoad(input, xy, 0).rgb;
        let dark_channel = min(pixel.r, min(pixel.g, pixel.b));
        result = u32(dark_channel * 255.0);
    }
    return result;
}

const half_group_size: u32 = 8u;

@compute
@workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(local_invocation_id) local_id: vec3<u32>) {
    let input_size = textureDimensions(input);
    if(global_id.x >= input_size.x || global_id.y >= input_size.y){
        return;
    }
    
    if (local_id.x < 4u && local_id.y < 4u) {
        local_patch.min_dark_channels[local_id.x][local_id.y] = 255u;
    }

    workgroupBarrier();

    let dark_channel = get_dark_channel(global_id.xy, input_size);
    let subpatch_x = 1 + i32(local_id.x / half_group_size);
    let subpatch_y = 1 + i32(local_id.y / half_group_size);
    atomicMin(&local_patch.min_dark_channels[subpatch_x][subpatch_y], dark_channel);

    workgroupBarrier();

    var delta: vec2<i32> = vec2(1, 1);
    if (local_id.x < half_group_size) {
        delta.x = -1;
    }
    if (local_id.y < half_group_size) {
        delta.y = -1;
    }

    var other_pixel_coord: vec2<u32>;
    var other_dark_channel: u32;
    var other_subpatch_x: i32;
    var other_subpatch_y: i32;
    
    other_pixel_coord = vec2<u32>(vec2<i32>(global_id.xy) + vec2(delta.x, 0) * i32(half_group_size));
    other_dark_channel = get_dark_channel(other_pixel_coord, input_size);
    other_subpatch_x =  subpatch_x + delta.x;
    other_subpatch_y =  subpatch_y + 0;
    atomicMin(&local_patch.min_dark_channels[other_subpatch_x][other_subpatch_y], other_dark_channel);

    other_pixel_coord = vec2<u32>(vec2<i32>(global_id.xy) + vec2(0, delta.y) * i32(half_group_size));
    other_dark_channel = get_dark_channel(other_pixel_coord, input_size);
    other_subpatch_x =  subpatch_x + 0;
    other_subpatch_y =  subpatch_y + delta.y;
    atomicMin(&local_patch.min_dark_channels[other_subpatch_x][other_subpatch_y], other_dark_channel);

    other_pixel_coord = vec2<u32>(vec2<i32>(global_id.xy) + vec2(delta.x, delta.y) * i32(half_group_size));
    other_dark_channel = get_dark_channel(other_pixel_coord, input_size);
    other_subpatch_x =  subpatch_x + delta.x;
    other_subpatch_y =  subpatch_y + delta.y;
    atomicMin(&local_patch.min_dark_channels[other_subpatch_x][other_subpatch_y], other_dark_channel);

    workgroupBarrier();

    let x = (f32(local_id.x) + 0.5) / f32(half_group_size) + 0.5;
    let y = (f32(local_id.y) + 0.5) / f32(half_group_size) + 0.5;

    let x0 = i32(floor(x));
    let x1 = x0 + 1;
    let y0 = i32(floor(y));
    let y1 = y0 + 1;

    let xf = x - f32(x0);
    let yf = y - f32(y0);

    let dark_00 = f32(local_patch.min_dark_channels[x0][y0]) / 255.0;
    let dark_01 = f32(local_patch.min_dark_channels[x0][y1]) / 255.0;
    let dark_10 = f32(local_patch.min_dark_channels[x1][y0]) / 255.0;
    let dark_11 = f32(local_patch.min_dark_channels[x1][y1]) / 255.0;

    let dark = dark_00 * (1.0 - xf) * (1.0 - yf) + dark_01 * (1.0 - xf) * (yf) + dark_10 * (xf) * (1.0 - yf) + dark_11 * xf * yf;
    
    var t = 1.0 - 0.95 * dark;
    t = max(t, 0.1);

    var c = textureLoad(input, global_id.xy, 0).rgb;

    let A = vec3(1.0);
    let dehazed = (c - A) / t + A;

    let coeff = params.value * 0.01;
    c = c * (1.0 - coeff) + dehazed * coeff;

    textureStore(output, global_id.xy, vec4<f32>(c, 1.0));
}