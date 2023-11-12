@group(0) @binding(0)
var input: texture_2d<f32>;

const num_bins: i32 = 255;

struct Airlight {
    estimated_airlight: f32,
    dark_channel_histogram: array<atomic<u32>, num_bins>,
}

@group(0) @binding(1)
var<storage, read_write> airlight: Airlight; 

@compute
@workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(local_invocation_id) local_id: vec3<u32>) {
    let input_size = textureDimensions(input);

    let num_pixels = f32(input_size.x * input_size.y);

    var cumulative = 1.0;
    for(var i: i32 = num_bins - 1; i >= 0; i -= 1) {
        cumulative -= f32(airlight.dark_channel_histogram[i]) / num_pixels;
        if (cumulative <= 0.99) {
            airlight.estimated_airlight = f32(i) / f32(num_bins - 1);
            break;
        }
    }
}

