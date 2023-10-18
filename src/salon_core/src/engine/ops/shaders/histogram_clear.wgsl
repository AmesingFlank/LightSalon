const max_bins:u32 = 256u;

struct Buffer {
    r: array<atomic<u32>, max_bins>,
    g: array<atomic<u32>, max_bins>,
    b: array<atomic<u32>, max_bins>,
    luma: array<atomic<u32>, max_bins>,
    num_bins: u32,
};

@group(0) @binding(0)
var<storage, read_write> buffer: Buffer;

@compute
@workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    buffer.r[i] = 0u;
    buffer.g[i] = 0u;
    buffer.b[i] = 0u;
    buffer.luma[i] = 0u;
}