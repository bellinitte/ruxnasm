mod anomalies;
mod instruction;
pub mod scanner;
mod span;
mod token;
pub mod tokenizer;
pub mod walker;

pub use anomalies::{Error, Warning};
pub use instruction::{Instruction, InstructionKind};
pub use span::{Location, Span, Spanned, Spanning};
pub use token::{Identifier, Token};
use walker::walk;

pub fn assemble(
    input_file_contents: impl AsRef<str>,
) -> Result<(Vec<u8>, Vec<Warning>), (Vec<Error>, Vec<Warning>)> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let words = match scanner::scan(input_file_contents.as_ref()) {
        Ok((words, new_warnings)) => {
            warnings.extend(new_warnings.iter().cloned().map(Warning::from));
            words
        }
        Err(error) => {
            errors.push(error.into());
            return Err((errors, warnings));
        }
    };

    let (new_errors, new_warnings) = walk(&words);
    errors.extend(new_errors);
    warnings.extend(new_warnings);

    Err((errors, warnings))
}
