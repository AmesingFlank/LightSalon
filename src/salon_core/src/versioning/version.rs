#[derive(Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn current_build() -> Self {
        Version {
            major: 0,
            minor: 1,
            patch: 0,
        }
    }
}