@group(0) @binding(0)
var input: texture_2d<f32>;
 
struct Buffer {
    sum_r: atomic<u32>,
    sum_g: atomic<u32>,
    sum_b: atomic<u32>,

    sum_count: atomic<u32>,
};

@group(0) @binding(1)
var<storage, read_write> buffer: Buffer;

var<workgroup> buffer_local: Buffer;

@compute
@workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(local_invocation_id) local_id: vec3<u32>) {
    let input_size = textureDimensions(input);

    let out_of_bounds = global_id.x >= input_size.x || global_id.y >= input_size.y;
    
    if (!out_of_bounds) {
        let c = textureLoad(input, global_id.xy, 0).rgb;
        let r = u32(c.r * 255.0);
        let g = u32(c.g * 255.0);
        let b = u32(c.b * 255.0);
        atomicAdd(&buffer_local.sum_r, r);
        atomicAdd(&buffer_local.sum_g, g);
        atomicAdd(&buffer_local.sum_b, b);
    }

    // this needs to be executed even for out-of-bounds threads;
    workgroupBarrier();

    if (!out_of_bounds) {
        if(local_id.x == 0u && local_id.y == 0u) {
            let wg_size = 16u * 16u;

            atomicAdd(&buffer.sum_r, buffer_local.sum_r / wg_size);
            atomicAdd(&buffer.sum_g, buffer_local.sum_g / wg_size);
            atomicAdd(&buffer.sum_b, buffer_local.sum_b / wg_size);

            atomicAdd(&buffer.sum_count, 1u);
        }
    }
}