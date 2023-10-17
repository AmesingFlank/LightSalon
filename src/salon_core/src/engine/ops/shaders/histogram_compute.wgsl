

@group(0) @binding(0)
var input: texture_2d<f32>;
 
struct Buffer {
    r: array<atomic<u32>, 256>,
    g: array<atomic<u32>, 256>,
    b: array<atomic<u32>, 256>,
    luma: array<atomic<u32>, 256>,
};

@group(0) @binding(1)
var<storage, read_write> buffer: Buffer;

@compute
@workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var c = textureLoad(input, global_id.xy, 0).rgb;
    c = clamp(c, vec3(0.0), vec3(1.0));
    let r = u32(c.r * 255.0);
    let g = u32(c.g * 255.0);
    let b = u32(c.b * 255.0);
    let luma = u32(dot(c, vec3(0.2126, 0.7152, 0.0722)));
    atomicAdd(&buffer.r[r], 1u);
    atomicAdd(&buffer.g[g], 1u);
    atomicAdd(&buffer.b[b], 1u);
    atomicAdd(&buffer.luma[luma], 1u);
}