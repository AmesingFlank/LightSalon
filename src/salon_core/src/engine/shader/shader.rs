use super::ShaderLibraryModule;

pub struct Shader {
    body_code: String,
    libraries: Vec<ShaderLibraryModule>
}

impl Shader {
    pub fn from_code(code: &str) -> Self {
        Shader {
            body_code:code.to_owned(),
            libraries: vec![]
        }
    }
}