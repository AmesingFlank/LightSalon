
// https://stackoverflow.com/questions/4200224/random-noise-functions-for-glsl

fn rand_f32_from_u32(u: u32) -> f32 {
    var x = u;
    x += ( x << 10u );
    x ^= ( x >>  6u );
    x += ( x <<  3u );
    x ^= ( x >> 11u );
    x += ( x << 15u );
    return f32(x) * (1.0 / f32(u32(0xffffffffu)));
}