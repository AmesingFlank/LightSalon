#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct ImageRating {
    pub num_stars: Option<u32>,
}

impl ImageRating {
    pub fn new(num_stars: Option<u32>) -> Self {
        Self { num_stars }
    }
}
