mod anomalies;
mod instruction;
pub(crate) mod scanner;
mod span;
mod token;
pub(crate) mod tokenizer;
pub(crate) mod walker;

pub use anomalies::{Error, Warning};
pub(crate) use instruction::{Instruction, InstructionKind};
pub(crate) use span::{Location, Span, Spanned, Spanning};
pub(crate) use token::{Identifier, Token};
use walker::walk;

/// Assembles an Uxn binary from a string representing an Uxntal program.
///
/// - In case the program is valid, returns an `Ok((Vec<u8>, Vec<Warning>))` &mdash; the binary
///   represented as a sequence of bytes in a `Vec`, along with any [`Warning`]s that have been
///   reported during the assembly.
/// - In case the program is invalid, i.e. it contains errors, returns an
///   `Err((Vec<Error>, Vec<Warning>))`, which contains all [`Error`]s in the program, along with
///   any [`Warning`]s that may have also been generated. The `Vec` containing the errors is always
///   non-empty.
pub fn assemble(
    source: impl AsRef<str>,
) -> Result<(Vec<u8>, Vec<Warning>), (Vec<Error>, Vec<Warning>)> {
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
