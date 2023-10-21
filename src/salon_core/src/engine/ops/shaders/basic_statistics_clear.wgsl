struct Buffer {
    mean_r: atomic<u32>,
    mean_g: atomic<u32>,
    mean_b: atomic<u32>,
};

@group(0) @binding(0)
var<storage, read_write> buffer: Buffer;

@compute
@workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    buffer.mean_r = 0u;
    buffer.mean_g = 0u;
    buffer.mean_b = 0u;
}