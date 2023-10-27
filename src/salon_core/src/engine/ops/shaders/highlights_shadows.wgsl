

@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var output: texture_storage_2d<rgba16float, write>;

struct Params {
    highlights: f32,
    shadows: f32,
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
    var XYZ = rgb_to_XYZ(rgb);
    var xyY = XYZ_to_xyY(XYZ);

    var Y = xyY.z;

    let highlights = params.highlights * 0.01;
    let shadows = params.shadows * 0.01;

    let highlights_threshold = 0.6;
    let shadows_threashold = 0.2;

    if(Y > highlights_threshold && highlights != 0.0) {
        let scale = 1.0 / (1.0 - highlights_threshold);
        Y = (Y - highlights_threshold) * scale;
        if(highlights < 0.0) {
            Y = pow(Y,  1.0 - highlights * 2.0);
        }
        else {
            Y = pow(Y, 1.0 / (1.0 + highlights * 2.0));
        }
        Y = Y / scale + highlights_threshold; 
    }

    if(Y < shadows_threashold && shadows != 0.0) {
        let scale = 1.0 / shadows_threashold;
        Y = Y * scale;
        if(shadows < 0.0) {
            Y = pow(Y,  1.0 - shadows);
        }
        else {
            Y = pow(Y, 1.0 / (1.0 + shadows));
        }
        Y = Y / scale; 
    }

    xyY.z = Y;
    XYZ = xyY_to_XYZ(xyY);
    rgb = XYZ_to_rgb(XYZ);
    textureStore(output, global_id.xy, vec4<f32>(rgb, 1.0));
}