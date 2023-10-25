

@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var output: texture_storage_2d<rgba16float, write>;

struct Params {
    temperature: f32,
    tint: f32
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
    var XYZ = rgb_to_XYZ(rgb);
    var xyY = XYZ_to_xyY(XYZ);
    var xy = xyY.xy;

    let CCT_clamp_min: f32 = 1000.0;
    let CCT_clamp_max: f32 = 25000.0;

    var CCT_Duv = xy_to_CCT_Duv(xy, CCT_clamp_min, CCT_clamp_max);

    var CCT = CCT_Duv.x;
    var Duv = CCT_Duv.y;

    if (params.temperature != 0.0) {
        CCT += params.temperature * 10.0;
        CCT = min(CCT, CCT_clamp_max);
        CCT = max(CCT, CCT_clamp_min);
    }

    if (params.tint != 0.0) {
        Duv -= params.tint / 3000.0;
    }

    CCT_Duv = vec2(CCT, Duv);

    xy = CCT_Duv_to_xy(CCT_Duv);
    xyY = vec3(xy, xyY.z);

    XYZ = xyY_to_XYZ(xyY);
    rgb = XYZ_to_rgb(XYZ);

    textureStore(output, global_id.xy, vec4(rgb, 1.0));
}