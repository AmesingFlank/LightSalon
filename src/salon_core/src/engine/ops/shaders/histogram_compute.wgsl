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

fn val_to_bin(v: f32) -> u32 {
    return u32(v * (f32(uniforms.num_bins) - 1.00001));
}

@compute
@workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    buffer.num_bins = uniforms.num_bins;

    var c = textureLoad(input, global_id.xy, 0).rgb;
    c = linear_to_srgb(c);
    c = clamp(c, vec3(0.0), vec3(1.0));
    let r_bin = val_to_bin(c.r);
    let g_bin = val_to_bin(c.g);
    let b_bin = val_to_bin(c.b);
    let luma_val = dot(c, vec3(0.2126, 0.7152, 0.0722));
    let luma_bin = val_to_bin(luma_val);
    atomicAdd(&buffer.r[r_bin], 1u);
    atomicAdd(&buffer.g[g_bin], 1u);
    atomicAdd(&buffer.b[b_bin], 1u);
    atomicAdd(&buffer.luma[luma_bin], 1u);
}