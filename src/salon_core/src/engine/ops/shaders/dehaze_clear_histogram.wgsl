const num_bins: i32 = 256;

struct Airlight {
    estimated_airlight: f32,
    dark_channel_histogram: array<atomic<u32>, num_bins>,
}

@group(0) @binding(0)
var<storage, read_write> airlight: Airlight;

@compute
@workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    airlight.dark_channel_histogram[i] = 0u;
}