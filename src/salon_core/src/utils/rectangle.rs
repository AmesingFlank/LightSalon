use super::vec::Vec2;

#[derive(Clone, PartialEq)]
pub struct Rectangle {
    pub min: Vec2<f32>,
    pub max: Vec2<f32>,
}