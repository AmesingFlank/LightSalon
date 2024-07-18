

@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var output: texture_storage_2d<rgba16float, write>;

struct Params {
    value: f32,
};

@group(0) @binding(2)
var<uniform> params: Params;

@compute
@workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let input_size = textureDimensions(input);
    if(global_id.x >= input_size.x || global_id.y >= input_size.y){
        return;
    }
    var rgb = textureLoad(input, global_id.xy, 0).rgb;

    var uv = vec2(f32(global_id.x) / f32(input_size.x - 1u), f32(global_id.y) / f32(input_size.y - 1u));

    let aspect_ratio = f32(input_size.x) / f32(input_size.y);
    // make the vignette a bit more round
    let x_factor = sqrt(aspect_ratio);
    uv.x *= x_factor;

    let inner_radius = 0.2;
    let outer_radius = 0.5 * max(1.0, x_factor) * 1.1;

    var strength = params.value * 0.01;
    strength = sign(strength) * sqrt(abs(strength));

    let dist = length(uv - vec2(0.5 * x_factor, 0.5));
    let vignette = smoothstep(inner_radius, outer_radius, dist) ;

    rgb *= 1.0 + vignette * strength;

    textureStore(output, global_id.xy, vec4<f32>(rgb, 1.0));
}