use super::{vec::Vec2};
use num::Num;
use serde;
use std::ops::{Mul};

// these are deliberately similar to WGSL
#[derive(Clone, Copy, PartialEq, Debug, serde::Deserialize, serde::Serialize)]
pub struct Mat2x2<T: Num> {
    row0: Vec2<T>,
    row1: Vec2<T>,
}

impl<T: Num + Copy> Mat2x2<T> {
    pub fn from_rows(row0: Vec2<T>, row1: Vec2<T>) -> Mat2x2<T> {
        Mat2x2 {
            row0, row1
        }
    }
}

impl<T: Num + Copy> Mul<Vec2<T>> for Mat2x2<T> {
    type Output = Vec2<T>;

    fn mul(self, v: Vec2<T>) -> Vec2<T> {
        Vec2 {
            x: v.dot(&self.row0),
            y: v.dot(&self.row1),
        }
    }
}
