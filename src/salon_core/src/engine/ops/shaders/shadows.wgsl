

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
@workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let input_size = textureDimensions(input);
    if(global_id.x >= input_size.x || global_id.y >= input_size.y){
        return;
    }
    var rgb = textureLoad(input, global_id.xy, 0).rgb;
    var hsl = rgb_to_hsl(rgb);

    let base = 0.2;
    let scale = 1.0 / base;
    let shadows = params.value * 0.01;
    var l = hsl.z;

    if(l < base) {
        l = l * scale;
        if(shadows < 0.0) {
            l = pow(l,  1.0 - shadows);
        }
        else {
            l = pow(l, 1.0 / (1.0 + shadows));
        }
        l = l / scale; 
    }

    hsl.z = l;
    rgb = hsl_to_rgb(hsl);
    textureStore(output, global_id.xy, vec4<f32>(rgb, 1.0));
}