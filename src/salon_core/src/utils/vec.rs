use num::Num;
use std::ops::{Add, Div, Mul, Sub};

// these are deliberately similar to WGSL
#[derive(Clone, Copy)]
pub struct Vec2<T: Num> {
    pub x: T,
    pub y: T,
}

pub fn vec2<T: Num>(xy: (T, T)) -> Vec2<T> {
    Vec2 { x: xy.0, y: xy.1 }
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
    pub x: T,
    pub y: T,
    pub z: T,
}

pub fn vec3<T: Num>(xyz: (T, T, T)) -> Vec3<T> {
    Vec3 {
        x: xyz.0,
        y: xyz.1,
        z: xyz.2,
    }
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
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

pub fn vec4<T: Num>(xyzw: (T, T, T, T)) -> Vec4<T> {
    Vec4 {
        x: xyzw.0,
        y: xyzw.1,
        z: xyzw.2,
        w: xyzw.3,
    }
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
