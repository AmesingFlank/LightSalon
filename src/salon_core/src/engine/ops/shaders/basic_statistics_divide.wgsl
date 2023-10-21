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
    result.sum_rgba.r = f32(working.sum_r / working.sum_count) / 255.0;
    result.sum_rgba.g = f32(working.sum_g / working.sum_count) / 255.0;
    result.sum_rgba.b = f32(working.sum_b / working.sum_count) / 255.0;
    result.sum_rgba.a = 1.0;
}