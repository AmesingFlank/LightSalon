use super::ShaderLibraryFunctions;

pub struct Shader {
    body_code: String,
    libraries: Vec<ShaderLibraryFunctions>
}

impl Shader {
    pub fn from_code(code: &str) -> Self {
        Shader {
            body_code:code.to_owned(),
            libraries: vec![]
        }
    }
}