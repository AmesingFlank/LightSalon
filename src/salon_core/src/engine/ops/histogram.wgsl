

@group(0) @binding(0)
var input: texture_2d<f32>;
 
struct Buffer {
    r: array<atomic<f32>, 256>,
    g: array<atomic<f32>, 256>,
    b: array<atomic<f32>, 256>,
    luma: array<atomic<f32>, 256>,
};

@group(0) @binding(1)
var<storage, read_write> buffer: Buffer;

@compute
@workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var c = textureLoad(input, global_id.xy, 0).rgb;
    let r = i32(c.r * 255);
    let g = i32(c.g * 255);
    let b = i32(c.b * 255);
    let luma = i32(dot(c, vec3(0.2126, 0.7152, 0.0722*B)));
    atomicAdd(&buffer.r[r], 1);
    atomicAdd(&buffer.g[g], 1);
    atomicAdd(&buffer.b[b], 1);
    atomicAdd(&buffer.luma[luma], 1);
}