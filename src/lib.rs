mod anomalies;
pub mod emitter;
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
) -> Result<(Vec<u8>, Vec<Warning>), (Vec<Warning>, Vec<Error>)> {
    let mut anomalies = Anomalies::new();

    let (tokens, tokenize_anomalies) = tokenizer::tokenize(input_file_contents);
    anomalies.extend(tokenize_anomalies);

    let (binary, emit_anomalies) = emitter::emit(&tokens);
    anomalies.extend(emit_anomalies);

    if anomalies.are_fatal() {
        return Err((
            anomalies.warnings().cloned().collect(),
            anomalies.errors().cloned().collect(),
        ));
    }

    println!("{:?}", binary);

    Ok((binary, anomalies.warnings().cloned().collect()))
}
