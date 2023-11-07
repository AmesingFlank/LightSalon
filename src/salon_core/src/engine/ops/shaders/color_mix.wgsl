

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
    return min(abs(ha - hb), min(abs(ha + 360.0 - hb), abs(ha - (hb + 360.0))));
}

@compute
@workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let input_size = textureDimensions(input);
    if(global_id.x >= input_size.x || global_id.y >= input_size.y){
        return;
    }
    var rgb = textureLoad(input, global_id.xy, 0).rgb;
    var hsl = rgb_to_LCh(rgb);

    var h = hsl.z;
    var s = hsl.y;
    var l = hsl.x;

    for(var i: i32 = 0; i < 8; i = i + 1) {
        let g = params.groups[i];
        let base_hue = f32(i) * (360.0 / 8.0);
        let diff = hue_diff(base_hue, h);
        let impact = max(0.0, 1.0 - diff / (360.0 / 8.0));
        
        let hue_shift = g.hue * (1.0 / 100.0) * (360.0 / 8.0) * impact;
        h += hue_shift;
        if (h > 360.0) {
            h -= 360.0;
        }
        if (h < 0.0) {
            h += 360.0;
        }

        let saturation_shift = 1.0 + g.saturation * (1.0 / 100.0) * impact;
        s *= saturation_shift;

        let lightness_shift = 1.0 + g.lightness * (1.0 / 100.0) * 0.5 * impact;
        l *= lightness_shift;
    }

    hsl.z = h;
    hsl.y = s;
    hsl.x = l;

    rgb = LCh_to_rgb(hsl);
    textureStore(output, global_id.xy, vec4(rgb, 1.0));
}