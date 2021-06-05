mod anomalies;
mod instruction;
pub(crate) mod scanner;
mod span;
mod token;
pub(crate) mod tokenizer;
pub(crate) mod walker;

pub use anomalies::{Error, Warning};
pub(crate) use instruction::{Instruction, InstructionKind};
pub use span::Span;
pub(crate) use span::{Location, Spanned, Spanning};
pub(crate) use token::{Identifier, Token};
use walker::walk;

pub type Result<T> = std::result::Result<(T, Vec<Warning>), (Vec<Error>, Vec<Warning>)>;

pub fn assemble(source: impl AsRef<str>) -> Result<Vec<u8>> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let words = match scanner::scan(source.as_ref()) {
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
