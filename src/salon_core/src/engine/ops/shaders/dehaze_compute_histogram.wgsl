@group(0) @binding(0)
var input: texture_2d<f32>;

const num_bins: i32 = 255;

struct Airlight {
    estimated_airlight: f32,
    dark_channel_histogram: array<atomic<u32>, num_bins>,
}

@group(0) @binding(1)
var<storage, read_write> airlight: Airlight;

var<workgroup> airlight_local: Airlight;


@compute
@workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(local_invocation_id) local_id: vec3<u32>) {
    let input_size = textureDimensions(input);

    let out_of_bounds = global_id.x >= input_size.x || global_id.y >= input_size.y;
    
    if (!out_of_bounds) {
        let c = textureLoad(input, global_id.xy, 0).rgb;
        let dark_channel = u32(min(min(c.r, c.g), min(c.b, 1.0)) * f32(num_bins - 1));

        atomicAdd(&airlight_local.dark_channel_histogram[dark_channel], 1u); 
    }

    // this needs to be executed even for out-of-bounds threads;
    workgroupBarrier();

    if (out_of_bounds) {
        return;
    }

    var write_index = local_id.x * 16u + local_id.y;
    while (write_index < u32(num_bins)){
        atomicAdd(&airlight.dark_channel_histogram[write_index], atomicLoad(&airlight_local.dark_channel_histogram[write_index])); 
        write_index = write_index + 16u * 16u;
    }
}

