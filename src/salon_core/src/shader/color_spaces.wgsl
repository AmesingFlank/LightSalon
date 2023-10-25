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
  hue = (hue / 6.0) % 1.0;
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

// matches Image.rs
const COLOR_SPACE_LINEAR: u32 = 0u;
const COLOR_SPACE_SRGB: u32 = 1u;

fn linear_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
  let srgb = pow(rgb, vec3(1.0 / 2.2));
  return srgb;
}

fn srgb_to_linear(srgb: vec3<f32>) -> vec3<f32> {
  let rgb = pow(srgb, vec3(2.2));
  return rgb;
}

// rgb (from sRGB) to CIE XYZ 1931
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

// CIE XYZ 1931 to rgb (from sRGB)
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

fn xy_to_CCT_Duv(xy: vec2<f32>) -> vec2<f32> {
  let uv = xy_to_uv(xy);

  // https://www.waveformlighting.com/tech/calculate-color-temperature-cct-from-cie-1931-xy-coordinates
  // let n = (xy.x - 0.3320) / (0.1858 - xy.y);
  // let CCT = 437.0 * n * n * n + 3601.0 * n * n + 6861.0 * n + 5517.0;

  // https://github.com/darktable-org/darktable/blob/99e1d8c3ba804474501c0fc8fdd26f722a492f6c/src/common/illuminants.h#L121
  // https://github.com/ofek/colour/blob/04f4863ef49093a93244c1fedafd1d5e2b1b76da/colour/temperature/cct.py#L836
  var n = (xy.x - 0.3366) / (xy.y - 0.1735);
  var CCT = (-949.86315 + 6253.80338 * exp(-n / 0.92159) + 28.70599f * exp(-n / 0.20039)  + 0.00004 * exp(-n / 0.07125));
  if(CCT > 50000.0) {
    n =  (xy.x - 0.3356) / (xy.y - 0.1691);
    CCT = 36284.48953 + 0.00228 * exp(-n / 0.07861) +  5.4535e-36 * exp(-n / 0.01543);
  }

  CCT = min(CCT, 25000.0);

  // we can also use uv_to_Duv here, but probably better to just compute Duv from distance to the locus
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