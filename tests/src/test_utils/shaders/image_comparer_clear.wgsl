struct Buffer {
    sum_squared_errors: atomic<u32>,
    count_squared_errors: atomic<u32>,
};

@group(0) @binding(0)
var<storage, read_write> buffer: Buffer;

@compute
@workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {   
    atomicStore(&buffer.sum_squared_errors, 0u);
    atomicStore(&buffer.count_squared_errors, 0u);
}