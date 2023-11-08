// matches Image.rs
const COLOR_SPACE_LINEAR_RGB: u32 = 0u;
const COLOR_SPACE_sRGB: u32 = 1u;
const COLOR_SPACE_HSL: u32 = 2u;
const COLOR_SPACE_LCh: u32 = 3u;
const COLOR_SPACE_HSLuv: u32 = 4u;

const LCh_HUE_RANGE: f32 = 6.2831853072; // 2.0 * PI;  use radians
const HSLuv_HUE_RANGE: f32 = 6.2831853072; // 2.0 * PI;  use radians
const HSL_HUE_RANGE: f32 = 1.0;

// conversion functions

fn to_linear_rgb(color: vec3<f32>, space: u32) -> vec3<f32> {
  if (space == COLOR_SPACE_LINEAR_RGB) {
    return color;
  }
  else if (space == COLOR_SPACE_sRGB) {
    return srgb_to_linear(color);
  }
  else if (space == COLOR_SPACE_HSL) {
    return hsl_to_rgb(color);
  }
  else if (space == COLOR_SPACE_LCh) {
    return LCh_to_rgb(color);
  }
  else if (space == COLOR_SPACE_HSLuv) {
    return hsluv_to_rgb(color);
  }
  else {
    return vec3(0.0);
  }
}

// https://en.wikipedia.org/wiki/HSL_and_HSV

// hmmc: hue, channel-wise min, Max and chroma (i.e. max-min)
fn rgb_to_hmmc(rgb: vec3<f32>) -> vec4<f32> {
  let M = max(rgb.r, max(rgb.g, rgb.b));
  let m = min(rgb.r, min(rgb.g, rgb.b));
  let chroma = M - m;
  let dc = vec3(rgb.g - rgb.b, rgb.b - rgb.r, rgb.r - rgb.g) / max(chroma, 0.001);
  var hue = dc.z + 4.0;
  hue = mix(hue, dc.y + 2.0, step(M, rgb.g));
  hue = mix(hue, dc.x, step(M, rgb.r));
  hue = hue / 6.0;
  if (hue < 0.0) {
    hue = hue + 1.0;
  }
  return vec4(hue, m, M, chroma);
}

fn hue_to_rgb(hue: f32) -> vec3<f32> {
  let r = abs(hue * 6.0 - 3.0) - 1.0;
  let g = -abs(hue * 6.0 - 2.0) + 2.0;
  let b = -abs(hue * 6.0 - 4.0) + 2.0;
  return clamp(vec3(r, g, b), vec3(0.0), vec3(1.0));
}

fn rgb_to_hsv(rgb: vec3<f32>) -> vec3<f32> {
  let hmmc = rgb_to_hmmc(rgb); 
  return vec3(hmmc.x, hmmc.w / max(hmmc.z, 0.001), hmmc.z);
}

fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
  let rgb = hue_to_rgb(hsv.x);
  return hsv.z * mix(vec3(1.0), rgb, hsv.y);
}

fn rgb_to_hsl(rgb: vec3<f32>) -> vec3<f32> {
  let hmmc = rgb_to_hmmc(rgb);
  let sum = hmmc.y + hmmc.z;
  let den = 1.0 - abs(sum - 1.0);
  return vec3(hmmc.x, hmmc.w / max(den, 0.001), sum * 0.5);
}

fn hsl_to_rgb(hsl: vec3<f32>) -> vec3<f32> {
  let rgb = hue_to_rgb(hsl.x);
  let chroma = (1.0 - abs(2.0 * hsl.z - 1.0)) * hsl.y;
  return chroma * (rgb - 0.5) + hsl.z;
}

fn linear_to_srgb_channel(C: f32) -> f32 {
  if (C <= 0.0031308) {
    return C * 12.92;
  }
  else{
    return 1.055 * pow(C, 1.0 / 2.4) - 0.055;
  }
}

fn srgb_to_linear_channel(C: f32) -> f32 {
  if (C <= 0.04045) {
    return C / 12.92;
  }
  else{
    return pow((C + 0.055) / 1.055, 2.4);
  }
}

fn linear_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
  let srgb = pow(rgb, vec3(1.0 / 2.2));
  return vec3(
    linear_to_srgb_channel(rgb.r),
    linear_to_srgb_channel(rgb.g),
    linear_to_srgb_channel(rgb.b)
  );
}

fn srgb_to_linear(srgb: vec3<f32>) -> vec3<f32> {
  return vec3(
    srgb_to_linear_channel(srgb.r),
    srgb_to_linear_channel(srgb.g),
    srgb_to_linear_channel(srgb.b)
  );
}

// rgb (linearized sRGB) to CIE XYZ 1931
fn rgb_to_XYZ(rgb: vec3<f32>) -> vec3<f32> {
  let column0 = vec3(
    0.4124564,
    0.2126729,
    0.0193339
  );
  let column1 = vec3(
    0.3575761,
    0.7151522,
    0.1191920
  );
  let column2 = vec3(
    0.1804375,
    0.0721750,
    0.9503041
  );
  let m = mat3x3(column0, column1, column2);
  return m * rgb;
}

// CIE XYZ 1931 to rgb (linearized sRGB)
fn XYZ_to_rgb(XYZ: vec3<f32>) -> vec3<f32> {
  let column0 = vec3(
    3.2404542,
    -0.9692660,
    0.0556434
  );
  let column1 = vec3(
    -1.5371385,
    1.8760108,
    -0.2040259
  );
  let column2 = vec3(
    -0.4985314,
    0.0415560,
    1.0572252
  );
  let m = mat3x3(column0, column1, column2);
  return (m * XYZ);
}

fn XYZ_to_xyY(XYZ: vec3<f32>) -> vec3<f32> {
  let X = XYZ.x;
  let Y = XYZ.y;
  let Z = XYZ.z;

  let x = X / (X + Y + Z);
  let y = Y / (X + Y + Z);
  return vec3(x, y, Y);
}

fn xyY_to_XYZ(xyY: vec3<f32>) -> vec3<f32> {
  let x = xyY.x;
  let y = xyY.y;
  let Y = xyY.z;

  let X = (Y / y) * x;
  let Z = (Y / y) * (1.0 - x - y);
  return vec3(X, Y, Z);
}

// https://google.github.io/filament/Filament.md.html
// https://cormusa.org/wp-content/uploads/2018/04/CORM_2011_Calculation_of_CCT_and_Duv_and_Practical_Conversion_Formulae.pdf

fn xy_to_uv(xy: vec2<f32>) -> vec2<f32> {
  let x = xy.x;
  let y = xy.y;
  let u = 4.0 * x / (-2.0 * x + 12.0 * y + 3.0);
  let v = 6.0 * y / (-2.0 * x + 12.0 * y + 3.0);
  return vec2(u, v);
}

fn uv_to_xy(uv: vec2<f32>) -> vec2<f32> {
  let u = uv.x;
  let v = uv.y;
  let x = 3.0 * u / (2.0 * u - 8.0 * v + 4.0);
  let y = 2.0 * v / (2.0 * u - 8.0 * v + 4.0);
  return vec2(x, y);
}

fn uv_to_Duv(uv: vec2<f32>) -> f32 {
  let u = uv.x;
  let v = uv.y;

  // https://www.waveformlighting.com/tech/calculate-duv-from-cie-1931-xy-coordinates/ 
  // https://cormusa.org/wp-content/uploads/2018/04/CORM_2011_Calculation_of_CCT_and_Duv_and_Practical_Conversion_Formulae.pdf

  let k6 = -0.00616793;
  let k5 = 0.0893944;
  let k4 = -0.5179722;
  let k3 = 1.5317403;
  let k2 = -2.4243787;
  let k1 = 1.925865;
  let k0 = -0.471106;
  let Lfp = sqrt((u - 0.292) * (u - 0.292) + (v - 0.24) * (v - 0.24));
  let a = acos((u - 0.292) / Lfp);
  let Lbb = k6 * pow(a, 6.0) + k5 * pow(a, 5.0) + k4 * pow(a, 4.0) + k3 * pow(a, 3.0) + k2 * pow(a, 2.0) + k1 * a + k0;
  let Duv = Lfp - Lbb;

  return Duv;
}

fn xy_to_CCT_McCamy(xy: vec2<f32>) -> f32 {
  // https://www.waveformlighting.com/tech/calculate-color-temperature-cct-from-cie-1931-xy-coordinates
  let n = (xy.x - 0.3320) / (0.1858 - xy.y);
  let CCT = 437.0 * n * n * n + 3601.0 * n * n + 6861.0 * n + 5517.0;
  return CCT;
}

fn xy_to_CCT_Hernandez(xy: vec2<f32>) -> f32 {
  // https://github.com/darktable-org/darktable/blob/99e1d8c3ba804474501c0fc8fdd26f722a492f6c/src/common/illuminants.h#L121
  // https://github.com/ofek/colour/blob/04f4863ef49093a93244c1fedafd1d5e2b1b76da/colour/temperature/cct.py#L836
  var n = (xy.x - 0.3366) / (xy.y - 0.1735);
  var CCT = (-949.86315 + 6253.80338 * exp(-n / 0.92159) + 28.70599 * exp(-n / 0.20039)  + 0.00004 * exp(-n / 0.07125));
  if(CCT > 50000.0) {
    n =  (xy.x - 0.3356) / (xy.y - 0.1691);
    CCT = 36284.48953 + 0.00228 * exp(-n / 0.07861) +  5.4535e-36 * exp(-n / 0.01543);
  }
  return CCT;
}

fn xy_to_CCT_Duv(xy: vec2<f32>) -> vec2<f32> {
  
  var CCT = xy_to_CCT_Hernandez(xy);

  // we can also use uv_to_Duv here, but probably better to just compute Duv from distance to the locus
  let uv = xy_to_uv(xy);
  let u0_v0 = T_planckian_to_uv_krystek(CCT);
  let Duv = length(u0_v0 - uv);

  return vec2(CCT, Duv);
}

// https://en.wikipedia.org/wiki/Planckian_locus
// https://google.github.io/filament/Filament.md.html
// Krystek's algorithm
// valid for 1000K < T < 15000K
fn T_planckian_to_uv_krystek(T: f32) -> vec2<f32> {
  let u = (0.860117757 + 1.54118254e-4 * T + 1.28641212e-7 * T * T) / (1.0 + 8.42420235e-4 * T + 7.08145163e-7 * T * T);
  let v = (0.317398726 + 4.22806245e-5 * T + 4.20481691e-8 * T * T) / (1.0 - 2.89741816e-5 * T + 1.61456053e-7 * T * T);
  return vec2(u, v); 
}

fn T_planckian_to_uv_kim(T: f32) -> vec2<f32> {
  return xy_to_uv(T_planckian_to_xy_kim(T)); 
}

fn T_planckian_to_uv_combined(T: f32) -> vec2<f32> {
  if(T < 15000.0){
    return T_planckian_to_uv_krystek(T);
  }
  else{
   return T_planckian_to_uv_kim(T);
  }
}

// https://en.wikipedia.org/wiki/Planckian_locus
// larger range than T_planckian_to_uv
// https://github.com/ddennedy/movit/blob/0b1705581552217b0e387bd687d65e4e3410ab91/white_balance_effect.cpp#L21 which implements the same formula
fn T_planckian_to_xy_kim(T: f32) -> vec2<f32> {
  let T_clamped = min(T, 25000.0);
	let invT = 1e3 / T_clamped;
  
  var x = 0.0;
  var y = 0.0;

	if (T <= 4000.0f) {
		x = ((-0.2661239 * invT - 0.2343589) * invT + 0.8776956) * invT + 0.179910;
	} else {
		x = ((-3.0258469 * invT + 2.1070379) * invT + 0.2226347) * invT + 0.240390;
	}

	if (T <= 2222.0f) {
		y = ((-1.1063814 * x - 1.34811020) * x + 2.18555832) * x - 0.20219683;
	} else if (T <= 4000.0f) {
		y = ((-0.9549476 * x - 1.37418593) * x + 2.09137015) * x - 0.16748867;
	} else {
		y = (( 3.0817580 * x - 5.87338670) * x + 3.75112997) * x - 0.37001483;
	}

  return vec2(x,y);
}

fn CCT_Duv_to_xy(CCT_Duv: vec2<f32>) -> vec2<f32> {
  // https://cormusa.org/wp-content/uploads/2018/04/CORM_2011_Calculation_of_CCT_and_Duv_and_Practical_Conversion_Formulae.pdf
  let CCT = CCT_Duv.x;
  let Duv = CCT_Duv.y;
  let u0_v0 = T_planckian_to_uv_krystek(CCT);
  let u1_v1 = T_planckian_to_uv_krystek(CCT + 1.0);
  let u0 = u0_v0.x;
  let v0 = u0_v0.y;
  let u1 = u1_v1.x;
  let v1 = u1_v1.y;
  let du = u0 - u1;
  let dv = v0 - v1;
  let sin_theta = dv / sqrt(du * du + dv * dv);
  let cos_theta = du / sqrt(du * du + dv * dv);
  let u = u0 - Duv * sin_theta;
  let v = v0 + Duv * cos_theta;
  return uv_to_xy(vec2(u, v));
}


// https://github.com/williammalo/hsluv-glsl/blob/master/hsluv-glsl.fsh
// prolly from https://en.wikipedia.org/wiki/CIELUV, with Yn = 1.0
fn Y_to_L(Y: f32) -> f32 {
  if (Y <= 0.0088564516790356308 /*(6/29)^3*/) {
    return Y * 903.2962962962963 /*(29/3)^3*/;
  }
  else {
    return 116.0 * pow(Y, 1.0 / 3.0) - 16.0;
  }
}

const REF_U = 0.19783000664283681;
const REF_V = 0.468319994938791;

fn L_to_Y(L: f32) -> f32 {
  if (L <= 8.0) {
    return L / 903.2962962962963;
  }
  else {
    return pow((L + 16.0) / 116.0, 3.0);
  }
}


fn XYZ_to_Luv(XYZ: vec3<f32>) -> vec3<f32>{
  let X = XYZ.x;
  let Y = XYZ.y;
  let Z = XYZ.z;

	let L = Y_to_L(Y);

	if (L == 0.0 || (X == 0.0 && Y == 0.0 && Z == 0.0)) {
		return vec3(0.0, 0.0, 0.0);
	}

	let u_prime = (4.0 * X) / (X + (15.0 * Y) + (3.0 * Z));
	let v_prime = (9.0 * Y) / (X + (15.0 * Y) + (3.0 * Z));
	let u = 13.0 * L * (u_prime - REF_U);
	let v = 13.0 * L * (v_prime - REF_V);

	return vec3(L, u, v);
}


fn Luv_to_XYZ(Luv: vec3<f32>) -> vec3<f32> {
  let L = Luv.x;
  let u = Luv.y;
  let v = Luv.z;

  if (L == 0.0) {
		return vec3(0.0, 0.0, 0.0);
	}

  let u_prime = u / (13.0 * L) + REF_U;
	let v_prime = v / (13.0 * L) + REF_V;

	let Y = L_to_Y(L);

	let X = Y * (9.0 * u_prime) / (4.0 * v_prime);
	let Z = Y * (12.0 - 3.0 * u_prime - 20.0 * v_prime) / (4.0 * v_prime);

	return vec3(X, Y, Z);
}

fn Luv_to_LCh(Luv: vec3<f32>) -> vec3<f32> {
  let L = Luv.x;
  let U = Luv.y;
  let V = Luv.z;

  let C = sqrt(U * U + V * V);

  var h = atan2(V, U);
  h = normalize_hue(h, LCh_HUE_RANGE);
  if (C < 1e-1) {
    h = 0.0;
  }
  
  return vec3(L, C, h);
}

fn LCh_to_Luv(LCh: vec3<f32>) -> vec3<f32> {
  let hrad = LCh.b;
  return vec3(
      LCh.r,
      cos(hrad) * LCh.g,
      sin(hrad) * LCh.g
  );
}

fn rgb_to_LCh(rgb: vec3<f32>) -> vec3<f32> {
  return (Luv_to_LCh(XYZ_to_Luv(rgb_to_XYZ(rgb))));
}

fn LCh_to_rgb(hsluv: vec3<f32>) -> vec3<f32> {
  return XYZ_to_rgb(Luv_to_XYZ(LCh_to_Luv((hsluv))));
}

fn normalize_hue(hue: f32, range: f32) -> f32 {
  var result = hue;
  if result < 0.0 {
    result += range;
  }
  if result > range {
    result -= range;
  }
  return result;
}


fn hsluv_to_LCh(hsluv: vec3<f32>) -> vec3<f32> {
  let L = hsluv.z;
  let saturation = hsluv.y;
  var hue = hsluv.x;

  if (saturation < 1e-3) {
    hue = 0.0;
  }

  let chroma = saturation * max_chroma_for_LH(L, hue) / 100.0;
  return vec3(L, chroma, hue);
}

fn LCh_to_hsluv(LCh: vec3<f32>) -> vec3<f32> {
  let L = LCh.x;
  let chroma = LCh.y;
  var hue = LCh.z;
  
  if (chroma < 1e-3) {
    hue = 0.0;
  }
  
  var saturation = chroma * 100.0 / max_chroma_for_LH(L, hue);
  if (L > 100.0 - 1e-3 || L < 1e-3){
    saturation = 0.0;
  }
  
  return vec3(hue, saturation, L);
}

fn length_of_ray_until_intersect(theta: f32, x: vec3<f32>, y: vec3<f32>) -> vec3<f32> {
    var len = y / (sin(theta) - x * cos(theta));
    if (len.r < 0.0) {len.r=1000.0;}
    if (len.g < 0.0) {len.g=1000.0;}
    if (len.b < 0.0) {len.b=1000.0;}
    return len;
} 

fn max_chroma_for_LH(L: f32, H: f32) -> f32 {
    let hrad = H;
    let m2 = mat3x3(
         3.2409699419045214  ,-0.96924363628087983 , 0.055630079696993609,
        -1.5373831775700935  , 1.8759675015077207  ,-0.20397695888897657 ,
        -0.49861076029300328 , 0.041555057407175613, 1.0569715142428786  
    );
    let sub1 = pow(L + 16.0, 3.0) / 1560896.0;
    var sub2: f32 = 0.0;
    if (sub1 > 0.0088564516790356308) {
      sub2 = sub1;
    } 
    else { 
      sub2 = L / 903.2962962962963;
    }

    let top1   = (284517.0 * m2[0] - 94839.0  * m2[2]) * sub2;
    let bottom = (632260.0 * m2[2] - 126452.0 * m2[1]) * sub2;
    let top2   = (838422.0 * m2[2] + 769860.0 * m2[1] + 731718.0 * m2[0]) * L * sub2;

    let bound0x = top1 / bottom;
    let bound0y = top2 / bottom;

    let bound1x = top1 / (bottom + 126452.0);
    let bound1y = (top2 - 769860.0 * L) / (bottom + 126452.0);

    let lengths0 = length_of_ray_until_intersect(hrad, bound0x, bound0y );
    let lengths1 = length_of_ray_until_intersect(hrad, bound1x, bound1y );

    return  min(lengths0.r,
            min(lengths1.r,
            min(lengths0.g,
            min(lengths1.g,
            min(lengths0.b,
                lengths1.b)))));
}

fn rgb_to_hsluv(rgb: vec3<f32>) -> vec3<f32> {
  return LCh_to_hsluv(rgb_to_LCh(rgb));
}

fn hsluv_to_rgb(hsluv: vec3<f32>) -> vec3<f32> {
  return LCh_to_rgb(hsluv_to_LCh(hsluv));
}

// interpolation functions

fn interpolate_color(color1: vec3<f32>, color2: vec3<f32>, t: f32, space: u32) -> vec3<f32> {
  if (space == COLOR_SPACE_LINEAR_RGB) {
    return interpolate_linear(color1, color2, t);
  }
  else if (space == COLOR_SPACE_sRGB) {
    return interpolate_srgb(color1, color2, t);
  }
  else if (space == COLOR_SPACE_HSL) {
    return interpolate_hsl(color1, color2, t);
  }
  else if (space == COLOR_SPACE_LCh) {
    return interpolate_LCh(color1, color2, t);
  }
  else if (space == COLOR_SPACE_HSLuv) {
    return interpolate_hsluv(color1, color2, t);
  }
  else { // fallback to linear interpolation
    return  mix(color1, color2, t);
  }
}


fn interpolate_linear(rgb1: vec3<f32>, rgb2: vec3<f32>, t: f32) -> vec3<f32> {
  return mix(rgb1, rgb2, t);
}

fn interpolate_srgb(srgb1: vec3<f32>, srgb2: vec3<f32>, t: f32) -> vec3<f32> {
  return linear_to_srgb(mix(srgb_to_linear(srgb1), srgb_to_linear(srgb2), t));
}

fn interpolate_hue(h1: f32, h2: f32, t: f32, hue_range: f32) -> f32 {
  if(abs(h1-h2) < hue_range * 0.5) {
    return mix(h1, h2, t);
  }
  var result: f32 = 0.0;
  if(h1 < h2) {
    result = mix(h1 + hue_range, h2, t);
  }
  else {
    result = mix(h1, h2 + hue_range, t);
  }
  result = normalize_hue(result, hue_range);

  return result;
}

fn interpolate_hsl(hsl1: vec3<f32>, hsl2: vec3<f32>, t: f32) -> vec3<f32> {
  let h = interpolate_hue(hsl1.x, hsl2.x, t, HSL_HUE_RANGE);

  return vec3(h, mix(hsl1.yz, hsl2.yz, t));
}

fn interpolate_LCh(LCh1: vec3<f32>, LCh2: vec3<f32>, t: f32) -> vec3<f32> {
  let h = interpolate_hue(LCh1.z, LCh2.z, t, LCh_HUE_RANGE);

  return vec3(mix(LCh1.xy, LCh2.xy, t), h);
}

fn interpolate_hsluv(hsluv1: vec3<f32>, hsluv2: vec3<f32>, t: f32) -> vec3<f32> {
  let h = interpolate_hue(hsluv1.x, hsluv2.x, t, HSLuv_HUE_RANGE);

  return vec3(h, mix(hsluv1.yz, hsluv2.yz, t));
}
