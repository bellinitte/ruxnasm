mod anomalies;
mod instruction;
mod span;
mod token;
pub mod tokenizer;

use anomalies::Anomalies;
pub use anomalies::{Error, Warning};
pub use instruction::{Instruction, InstructionKind};
pub use span::{Location, Span, Spanned, Spanning};
pub use token::{Identifier, Token};

pub fn assemble(
    input_file_contents: &str,
) -> Result<([u8; 256 * 256], Vec<Warning>), (Vec<Warning>, Vec<Error>)> {
    let (tokens, tokenize_anomalies) = tokenizer::tokenize(input_file_contents);
    if tokenize_anomalies.are_fatal() {
        return Err((
            tokenize_anomalies.warnings().cloned().collect(),
            tokenize_anomalies.errors().cloned().collect(),
        ));
    }
    println!(
        "{:#?}",
        tokens
            .into_iter()
            .map(|Spanned { node, .. }: Spanned<Token>| node)
            .collect::<Vec<Token>>()
    );

    Ok((
        [0; 256 * 256],
        tokenize_anomalies.warnings().cloned().collect(),
    ))
}
