

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
    var uv = xy_to_uv(xy);

    let original_CCT = 5000.0;
    let original_CCT_uv = T_planckian_to_uv_krystek(original_CCT);

    var new_CCT = original_CCT;

    if (params.temperature != 0.0) {
        new_CCT = original_CCT + params.temperature * 10.0;
    }

    let new_CCT_uv = T_planckian_to_uv_krystek(new_CCT);
    var new_CCT_uv_delta = uv - new_CCT_uv;

    if (params.tint != 0.0) {
        let change_in_uv = vec2(1.0, -1.0) * params.tint / 3000.0;
        new_CCT_uv_delta = new_CCT_uv_delta + change_in_uv;
    }

    uv = original_CCT_uv + new_CCT_uv_delta;

    xy = uv_to_xy(uv);
    xyY = vec3(xy, xyY.z);

    XYZ = xyY_to_XYZ(xyY);
    rgb = XYZ_to_rgb(XYZ);

    textureStore(output, global_id.xy, vec4(rgb, 1.0));
}