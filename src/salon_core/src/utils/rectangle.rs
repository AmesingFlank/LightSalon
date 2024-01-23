use super::vec::Vec2;
use serde;

#[derive(Clone, Copy, PartialEq, Debug, serde::Deserialize, serde::Serialize)]
pub struct Rectangle {
    pub min: Vec2<f32>,
    pub max: Vec2<f32>,
}
