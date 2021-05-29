pub mod instruction;
pub mod span;

pub use instruction::*;
pub use span::*;

pub fn assemble(_source_contents: &str) -> Result<Vec<u8>, ()> {
    todo!()
}
