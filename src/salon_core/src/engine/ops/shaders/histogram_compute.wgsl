const max_bins:u32 = 256u;

@group(0) @binding(0)
var input: texture_2d<f32>;

struct Uniforms {
    num_bins: u32,
};

@group(0) @binding(1)
var<uniform> uniforms: Uniforms;
 
struct Buffer {
    r: array<atomic<u32>, max_bins>,
    g: array<atomic<u32>, max_bins>,
    b: array<atomic<u32>, max_bins>,
    luma: array<atomic<u32>, max_bins>,
    num_bins: u32,
};

@group(0) @binding(2)
var<storage, read_write> buffer: Buffer;

var<workgroup> buffer_local: Buffer;

fn val_to_bin(v: f32) -> u32 {
    return u32(v * (f32(uniforms.num_bins) - 1.0));
}

@compute
@workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(local_invocation_id) local_id: vec3<u32>) {
    let input_size = textureDimensions(input);

    let out_of_bounds = global_id.x >= input_size.x || global_id.y >= input_size.y;

    if (!out_of_bounds) {
        buffer.num_bins = uniforms.num_bins;

        var c = textureLoad(input, global_id.xy, 0).rgb;
        c = linear_to_srgb(c);
        c = clamp(c, vec3(0.0), vec3(1.0));

        let jitter_seed = global_id.x * input_size.y + global_id.y;
        let jitter = (rand_f32_from_u32(jitter_seed) - 0.5) * (1.0 / 255.5);

        let r_bin = val_to_bin(c.r + jitter);
        let g_bin = val_to_bin(c.g + jitter);
        let b_bin = val_to_bin(c.b + jitter);
        let luma_bin = val_to_bin(dot(c, vec3(0.2126, 0.7152, 0.0722)) + jitter);

        atomicAdd(&buffer_local.r[r_bin], 1u);
        atomicAdd(&buffer_local.g[g_bin], 1u);
        atomicAdd(&buffer_local.b[b_bin], 1u);
        atomicAdd(&buffer_local.luma[luma_bin], 1u);
    }

    // this needs to be executed even for out-of-bounds threads;
    workgroupBarrier();

    if (!out_of_bounds) {
        var write_index = local_id.x * 16u + local_id.y;
        while (write_index < max_bins){
            atomicAdd(&buffer.r[write_index], atomicLoad(&buffer_local.r[write_index]));
            atomicAdd(&buffer.g[write_index], atomicLoad(&buffer_local.g[write_index]));
            atomicAdd(&buffer.b[write_index], atomicLoad(&buffer_local.b[write_index]));
            atomicAdd(&buffer.luma[write_index], atomicLoad(&buffer_local.luma[write_index]));
            write_index = write_index + 16u * 16u;
        }
    }
}