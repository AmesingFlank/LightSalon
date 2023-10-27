use num::Num;
use std::ops::{Add, Div, Mul, Sub};

// these are deliberately similar to WGSL
#[derive(Clone, Copy)]
pub struct Vec2<T: Num> {
    x: T,
    y: T,
}

pub fn vec2<T: Num>(x: T, y: T) -> Vec2<T> {
    Vec2 { x, y }
}

// ... your existing Vec2, Vec3, and Vec4 definitions ...

impl<T: Num + Copy> Add for Vec2<T> {
    type Output = Vec2<T>;

    fn add(self, other: Vec2<T>) -> Vec2<T> {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: Num + Copy> Sub for Vec2<T> {
    type Output = Vec2<T>;

    fn sub(self, other: Vec2<T>) -> Vec2<T> {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T: Num + Copy> Mul for Vec2<T> {
    type Output = Vec2<T>;

    fn mul(self, other: Vec2<T>) -> Vec2<T> {
        Vec2 {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl<T: Num + Copy> Div for Vec2<T> {
    type Output = Vec2<T>;

    fn div(self, other: Vec2<T>) -> Vec2<T> {
        Vec2 {
            x: self.x / other.x,
            y: self.y / other.y,
        }
    }
}

impl<T: Num + Copy> Mul<T> for Vec2<T> {
    type Output = Vec2<T>;

    fn mul(self, scalar: T) -> Vec2<T> {
        Vec2 {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl<T: Num + Copy> Div<T> for Vec2<T> {
    type Output = Vec2<T>;

    fn div(self, scalar: T) -> Vec2<T> {
        Vec2 {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Vec3<T: Num> {
    x: T,
    y: T,
    z: T,
}

pub fn vec3<T: Num>(x: T, y: T, z: T) -> Vec3<T> {
    Vec3 { x, y, z }
}

impl<T: Num + Copy> Add for Vec3<T> {
    type Output = Vec3<T>;

    fn add(self, other: Vec3<T>) -> Vec3<T> {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl<T: Num + Copy> Sub for Vec3<T> {
    type Output = Vec3<T>;

    fn sub(self, other: Vec3<T>) -> Vec3<T> {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl<T: Num + Copy> Mul for Vec3<T> {
    type Output = Vec3<T>;

    fn mul(self, other: Vec3<T>) -> Vec3<T> {
        Vec3 {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

impl<T: Num + Copy> Div for Vec3<T> {
    type Output = Vec3<T>;

    fn div(self, other: Vec3<T>) -> Vec3<T> {
        Vec3 {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
        }
    }
}

impl<T: Num + Copy> Mul<T> for Vec3<T> {
    type Output = Vec3<T>;

    fn mul(self, scalar: T) -> Vec3<T> {
        Vec3 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl<T: Num + Copy> Div<T> for Vec3<T> {
    type Output = Vec3<T>;

    fn div(self, scalar: T) -> Vec3<T> {
        Vec3 {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Vec4<T: Num> {
    x: T,
    y: T,
    z: T,
    w: T,
}

pub fn vec4<T: Num>(x: T, y: T, z: T, w: T) -> Vec4<T> {
    Vec4 { x, y, z, w }
}

// ... your existing Vec2 and Vec3 definitions ...

impl<T: Num + Copy> Add for Vec4<T> {
    type Output = Vec4<T>;

    fn add(self, other: Vec4<T>) -> Vec4<T> {
        Vec4 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl<T: Num + Copy> Sub for Vec4<T> {
    type Output = Vec4<T>;

    fn sub(self, other: Vec4<T>) -> Vec4<T> {
        Vec4 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl<T: Num + Copy> Mul for Vec4<T> {
    type Output = Vec4<T>;

    fn mul(self, other: Vec4<T>) -> Vec4<T> {
        Vec4 {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
            w: self.w * other.w,
        }
    }
}

impl<T: Num + Copy> Div for Vec4<T> {
    type Output = Vec4<T>;

    fn div(self, other: Vec4<T>) -> Vec4<T> {
        Vec4 {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
            w: self.w / other.w,
        }
    }
}

impl<T: Num + Copy> Mul<T> for Vec4<T> {
    type Output = Vec4<T>;

    fn mul(self, scalar: T) -> Vec4<T> {
        Vec4 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
            w: self.w * scalar,
        }
    }
}

impl<T: Num + Copy> Div<T> for Vec4<T> {
    type Output = Vec4<T>;

    fn div(self, scalar: T) -> Vec4<T> {
        Vec4 {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
            w: self.w / scalar,
        }
    }
}

pub fn dot_vec4<T: Num>(a: Vec4<T>, b: Vec4<T>) -> T {
    a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w
}
