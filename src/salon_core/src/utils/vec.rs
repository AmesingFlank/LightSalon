use super::math;
use num::Num;
use serde;
use std::ops::{Add, Div, Mul, Sub};

// these are deliberately similar to WGSL
#[derive(Clone, Copy, PartialEq, Debug, serde::Deserialize, serde::Serialize)]
pub struct Vec2<T: Num> {
    pub x: T,
    pub y: T,
}

pub fn vec2<T: Num>((x, y): (T, T)) -> Vec2<T> {
    Vec2 { x, y }
}

impl<T: Num + Copy> Vec2<T> {
    pub fn xy(&self) -> (T, T) {
        (self.x, self.y)
    }

    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y
    }

    // https://stackoverflow.com/questions/563198/how-do-you-detect-where-two-line-segments-intersect/565282#565282
    pub fn cross(&self, other: &Self) -> T {
        self.x * other.y - self.y * other.x
    }
}

impl Vec2<f32> {
    pub fn length(&self) -> f32 {
        self.dot(self).sqrt()
    }
    pub fn normalized(&self) -> Self {
        *self / self.length()
    }
    pub fn clamp(&self, min: &Self, max: &Self) -> Self {
        Self {
            x: math::clamp(self.x, min.x, max.x),
            y: math::clamp(self.y, min.y, max.y),
        }
    }
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

#[derive(Clone, Copy, PartialEq)]
pub struct Vec3<T: Num> {
    pub x: T,
    pub y: T,
    pub z: T,
}

pub fn vec3<T: Num>((x, y, z): (T, T, T)) -> Vec3<T> {
    Vec3 { x, y, z }
}

impl<T: Num + Copy> Vec3<T> {
    pub fn xyz(&self) -> (T, T, T) {
        (self.x, self.y, self.z)
    }

    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl Vec3<f32> {
    pub fn length(&self) -> f32 {
        self.dot(self).sqrt()
    }
    pub fn normalized(&self) -> Self {
        *self / self.length()
    }
    pub fn clamp(&self, min: &Self, max: &Self) -> Self {
        Self {
            x: math::clamp(self.x, min.x, max.x),
            y: math::clamp(self.y, min.y, max.y),
            z: math::clamp(self.z, min.z, max.z),
        }
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

#[derive(Clone, Copy, PartialEq)]
pub struct Vec4<T: Num> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

pub fn vec4<T: Num>((x, y, z, w): (T, T, T, T)) -> Vec4<T> {
    Vec4 { x, y, z, w }
}

impl<T: Num + Copy> Vec4<T> {
    pub fn xyzw(&self) -> (T, T, T, T) {
        (self.x, self.y, self.z, self.w)
    }

    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }
}

impl Vec4<f32> {
    pub fn length(&self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalized(&self) -> Self {
        *self / self.length()
    }
    pub fn clamp(&self, min: &Self, max: &Self) -> Self {
        Self {
            x: math::clamp(self.x, min.x, max.x),
            y: math::clamp(self.y, min.y, max.y),
            z: math::clamp(self.z, min.z, max.z),
            w: math::clamp(self.w, min.w, max.w),
        }
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
