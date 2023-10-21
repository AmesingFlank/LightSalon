@group(0) @binding(0)
var input: texture_2d<f32>;
 
struct Buffer {
    mean_r: atomic<u32>,
    mean_g: atomic<u32>,
    mean_b: atomic<u32>,
};

@group(0) @binding(1)
var<storage, read_write> buffer: Buffer;

var<workgroup> buffer_local: Buffer;

@compute
@workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(local_invocation_id) local_id: vec3<u32>) {
    let input_size = textureDimensions(input);
    if(global_id.x >= input_size.x || global_id.y >= input_size.y){
        return;
    }
    
    let c = textureLoad(input, global_id.xy, 0).rgb;
    let r = u32(c.r * 255.0);
    let g = u32(c.g * 255.0);
    let b = u32(c.b * 255.0);
    atomicAdd(&buffer_local.mean_r, r);
    atomicAdd(&buffer_local.mean_r, g);
    atomicAdd(&buffer_local.mean_r, b);

    workgroupBarrier();

    if(local_id.x == 0u && local_id.y == 0u) {
        atomicAdd(&buffer.mean_r, buffer_local.mean_r);
        atomicAdd(&buffer.mean_g, buffer_local.mean_g);
        atomicAdd(&buffer.mean_b, buffer_local.mean_b);
    }
}