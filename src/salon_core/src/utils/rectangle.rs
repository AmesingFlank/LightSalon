use super::vec::{vec2, Vec2};
use serde;

#[derive(Clone, Copy, PartialEq, Debug, serde::Deserialize, serde::Serialize)]
pub struct Rectangle {
    pub center: Vec2<f32>,
    pub size: Vec2<f32>,
}

impl Rectangle {
    pub fn regular() -> Self {
        Self {
            center: vec2((0.5, 0.5)),
            size: vec2((1.0, 1.0)),
        }
    }

    pub fn min(&self) -> Vec2<f32> {
        self.center - self.size * 0.5
    }
    pub fn max(&self) -> Vec2<f32> {
        self.center + self.size * 0.5
    }
    pub fn from_min_max(min: Vec2<f32>, max: Vec2<f32>) -> Self {
        Self {
            center: (min + max) * 0.5,
            size: max - min,
        }
    }
}
