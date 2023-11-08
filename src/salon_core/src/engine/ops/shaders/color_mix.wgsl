

@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var output: texture_storage_2d<rgba16float, write>;

struct Group {
    hue: f32,
    saturation: f32,
    lightness: f32,
    padding: f32,
};

struct Params {
    groups: array<Group, 8>,
};

@group(0) @binding(2)
var<uniform> params: Params;

fn hue_diff(ha: f32, hb: f32) -> f32 {
    return min(abs(ha - hb), min(abs(ha + LCh_HUE_RANGE - hb), abs(ha - (hb + LCh_HUE_RANGE))));
}

@compute
@workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let input_size = textureDimensions(input);
    if(global_id.x >= input_size.x || global_id.y >= input_size.y){
        return;
    }
    var rgb = textureLoad(input, global_id.xy, 0).rgb;
    var hsl = rgb_to_hsluv(rgb);

    var h = hsl.x;
    var s = hsl.y;
    var l = hsl.z;

    let group_hue_range = (HSLuv_HUE_RANGE / 8.0);

    for(var i: i32 = 0; i < 8; i = i + 1) {
        let g = params.groups[i];
        
        let base_hue = f32(i) * group_hue_range;
        let diff = hue_diff(base_hue, h);
        let impact = max(0.0, 1.0 - diff / group_hue_range);
        
        let hue_shift = g.hue * (1.0 / 100.0) * group_hue_range * impact;
        h += hue_shift;
        h = normalize_hue(h, HSLuv_HUE_RANGE);

        let saturation_shift = 1.0 + g.saturation * (1.0 / 100.0) * impact;
        s *= saturation_shift;

        let lightness_shift = 1.0 + g.lightness * (1.0 / 100.0) * 0.5 * impact;
        l *= lightness_shift;
    }

    hsl.x = h;
    hsl.y = s;
    hsl.z = l;

    rgb = hsluv_to_rgb(hsl);
    textureStore(output, global_id.xy, vec4(rgb, 1.0));
}