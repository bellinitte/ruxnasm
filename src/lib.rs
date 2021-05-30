mod error;
pub mod instruction;
pub mod span;
mod token;
pub mod tokenizer;

pub use error::Error;
pub use instruction::{Instruction, InstructionKind};
pub use span::{Location, Span, Spanned, Spanning};
pub use token::{Identifier, Token};

pub fn assemble(input_file_contents: &str) -> Result<[u8; 256 * 256], Vec<Error>> {
    let tokens = tokenizer::tokenize(input_file_contents)
        .map_err(|errs| errs.into_iter().map(Error::from).collect::<Vec<Error>>())?;
    println!(
        "{:#?}",
        tokens
            .into_iter()
            .map(|Spanned { node, .. }: Spanned<Token>| node)
            .collect::<Vec<Token>>()
    );

    Ok([0; 256 * 256])
}
