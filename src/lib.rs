mod anomalies;
pub(crate) mod emitter;
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
    let mut warnings = Vec::new();

    let words = match scanner::scan(source.as_ref()) {
        Ok((words, new_warnings)) => {
            warnings.extend(new_warnings);
            words
        }
        Err(error) => {
            return Err((vec![error], warnings));
        }
    };

    let (statements, definitions) = match walker::walk(words) {
        Ok((statements, definitions, new_warnings)) => {
            warnings.extend(new_warnings);
            (statements, definitions)
        }
        Err((errors, new_warnings)) => {
            warnings.extend(new_warnings);
            return Err((errors, warnings));
        }
    };

    // println!("statements: {:#?}", statements);
    // println!("labels: {:?}", definitions.labels.keys());
    // println!("sublabels: {:?}", definitions.sublabels.keys());

    match emitter::emit(statements, definitions) {
        Ok((binary, new_warnings)) => {
            warnings.extend(new_warnings);
            Ok((binary, warnings))
        }
        Err((errors, new_warnings)) => {
            warnings.extend(new_warnings);
            Err((errors, warnings))
        }
    }
}
