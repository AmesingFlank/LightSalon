use super::vec::Vec2;

#[derive(Clone, PartialEq, Debug)]
pub struct Rectangle {
    pub min: Vec2<f32>,
    pub max: Vec2<f32>,
}