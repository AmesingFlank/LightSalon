#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ShaderLibraryModule {
    ColorSpaces,
    Random,
}

pub struct ShaderLibrary {

}

impl ShaderLibrary {
    pub fn get_library_module_code(functions: ShaderLibraryModule) -> &'static str {
        match functions {
            ShaderLibraryModule::ColorSpaces => {
                include_str!("./color_spaces.wgsl")
            }
            ShaderLibraryModule::Random => {
                include_str!("./random.wgsl")
            }
        }
    }
}