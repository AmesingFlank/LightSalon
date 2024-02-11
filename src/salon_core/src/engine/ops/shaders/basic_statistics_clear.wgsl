struct Buffer {
    sum_r: atomic<u32>,
    sum_g: atomic<u32>,
    sum_b: atomic<u32>,

    sum_count: atomic<u32>,
};

@group(0) @binding(0)
var<storage, read_write> buffer: Buffer;

@compute
@workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    atomicStore(&buffer.sum_r, 0u);
    atomicStore(&buffer.sum_g, 0u);
    atomicStore(&buffer.sum_b, 0u);
    atomicStore(&buffer.sum_count, 0u);
}