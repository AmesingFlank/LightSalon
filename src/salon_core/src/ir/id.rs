pub type Id = i32;


#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum IdTag {
    Output,
    DataForEditor,
}