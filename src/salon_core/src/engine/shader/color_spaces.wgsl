// https://www.shadertoy.com/view/XljGzV

fn rgb_to_hsv(c: vec3<f32>) -> vec3<f32> {
    let K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    let p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    let q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    let d = q.x - min(q.w, q.y);
    let e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

fn hsv_to_rgb(c: vec3<f32>) -> vec3<f32> {
    let K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3(0.0), vec3(1.0)), c.y);
}

fn hsl_to_rgb(c: vec3<f32>) -> vec3<f32> {
    let rgb = clamp(abs((vec3(c.x * 6.0) + vec3(0.0, 4.0, 2.0)) % 6.0 - 3.0) - 1.0, vec3(0.0), vec3(1.0));
    return c.z + c.y * (rgb - 0.5) * (1.0 - abs(2.0 * c.z - 1.0));
} 

fn rgb_to_hsl(c: vec3<f32>) -> vec3<f32> {
    var h = 0.0;
	var s = 0.0;
	var l = 0.0;
	var r = c.r;
	var g = c.g;
	var b = c.b;
	var cMin = min( r, min( g, b ) );
	var cMax = max( r, max( g, b ) );

	l = ( cMax + cMin ) / 2.0;
	if ( cMax > cMin ) {
		var cDelta = cMax - cMin;

        if (l < 0.0) {
            s = cDelta / ( cMax + cMin );
        }
        else {
            s = cDelta / ( 2.0 - ( cMax + cMin ) );
        } 
        
		if ( r == cMax ) {
			h = ( g - b ) / cDelta;
		} else if ( g == cMax ) {
			h = 2.0 + ( b - r ) / cDelta;
		} else {
			h = 4.0 + ( r - g ) / cDelta;
		}
		if ( h < 0.0) {
			h += 6.0;
		}
		h = h / 6.0;
	}
	return vec3( h, s, l );
}