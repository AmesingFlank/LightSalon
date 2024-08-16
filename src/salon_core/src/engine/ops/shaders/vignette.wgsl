

@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var output: texture_storage_2d<rgba16float, write>;

struct Params {
    vignette: f32,
    midpoint: f32,
    feather: f32,
    roundness: f32,
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
    let x_factor = mix(1.0, aspect_ratio, params.roundness * 0.01);
    uv.x *= x_factor;

    let min_midpoint = 0.5 * 0.75;
    let max_midpoint = 0.5 * 1.25;
    let midpoint = mix(min_midpoint, max_midpoint, params.midpoint * 0.01);

    var delta = params.vignette * 0.01;
    delta = sign(delta) * sqrt(abs(delta));
    
    var feather = params.feather * 0.01;

    var dist = length(uv - vec2(0.5 * x_factor, 0.5));
    dist = dist - midpoint;
    
    // sigmoid function
    var coeff = 1.0 / (1.0 + exp(-1.0 * dist * (1.01 - pow(feather, 0.05)) * 500.0));
    coeff = sqrt(sqrt(coeff));
    var hsv = rgb_to_hsv(rgb);

    hsv.z *= 1.0 + coeff * delta;

    hsv.y *= inverseSqrt(1.0 + coeff * delta * hsv.y);
    hsv.y = min(hsv.y, 1.0);

    rgb = hsv_to_rgb(hsv);

    textureStore(output, global_id.xy, vec4<f32>(rgb, 1.0));
}