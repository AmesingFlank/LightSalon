pub enum ShaderLibraryFunctions {
    ColorSpaces
}

pub struct ShaderLibrary {

}

impl ShaderLibrary {
    pub fn get_library_functions_code(functions: ShaderLibraryFunctions) -> &'static str {
        match functions {
            ShaderLibraryFunctions::ColorSpaces => {
                include_str!("./color_spaces.wgsl")
            }
        }
    }
}