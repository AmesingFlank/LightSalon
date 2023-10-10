use super::{ShaderLibraryModule, ShaderLibrary};

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
    pub fn with_library(mut self, lib: ShaderLibraryModule) -> Self {
        self.libraries.push(lib);
        self
    }
    pub fn full_code(&self) -> String {
        let mut libraries_code = String::new();
        for l in self.libraries.iter() {
            libraries_code = libraries_code + ShaderLibrary::get_library_module_code(*l) + "\n";
        }
        return libraries_code + self.body_code.as_str();
    }
}