struct Buffer {
    sum_squared_errors: atomic<u32>,
    count_squared_errors: atomic<u32>,
};

@group(0) @binding(0)
var<storage, read_write> buffer: Buffer;

@group(0) @binding(1)
var lhs: texture_2d<f32>;
 
@group(0) @binding(2)
var rhs: texture_2d<f32>;

var<workgroup> buffer_local: Buffer;

const error_factor:f32 = 255.0;

@compute
@workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(local_invocation_id) local_id: vec3<u32>) {
    let input_size = textureDimensions(lhs);

    let out_of_bounds = global_id.x >= input_size.x || global_id.y >= input_size.y;

    if (!out_of_bounds) {
        let l = textureLoad(lhs, global_id.xy, 0).rgb;
        let r = textureLoad(rhs, global_id.xy, 0).rgb;
        
        let error = l - r;
        let squared_error = dot(error * error, vec3(1.0)) / 3.0;
        let squared_error_u32 = u32(squared_error * error_factor);

        atomicAdd(&buffer_local.sum_squared_errors, squared_error_u32);
        atomicAdd(&buffer_local.count_squared_errors, 1u);
    }

    // this needs to be executed even for out-of-bounds threads;
    workgroupBarrier();

    if (local_id.x == 0u && local_id.y == 0u) {
        atomicAdd(&buffer.sum_squared_errors,  atomicLoad(&buffer_local.sum_squared_errors));
        atomicAdd(&buffer.count_squared_errors,  atomicLoad(&buffer_local.count_squared_errors));
    }
}