struct WorkingBuffer {
    sum_r: atomic<u32>,
    sum_g: atomic<u32>,
    sum_b: atomic<u32>,

    sum_count: atomic<u32>,
};

@group(0) @binding(0)
var<storage, read_write> working: WorkingBuffer;

struct ResultBuffer {
    sum_rgba: vec4<f32>,
};

@group(0) @binding(1)
var<storage, read_write> result: ResultBuffer;

@compute
@workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let sum_r = atomicLoad(&working.sum_r);
    let sum_g = atomicLoad(&working.sum_g);
    let sum_b = atomicLoad(&working.sum_b);
    let sum_count = atomicLoad(&working.sum_count);
    result.sum_rgba.r = f32(sum_r / sum_count) / 255.0;
    result.sum_rgba.g = f32(sum_g / sum_count) / 255.0;
    result.sum_rgba.b = f32(sum_b / sum_count) / 255.0;
    result.sum_rgba.a = 1.0;
}