mod anomalies;
pub(crate) mod emitter;
mod instruction;
pub(crate) mod scanner;
mod span;
mod token;
pub(crate) mod tokenizer;
mod traits;
pub(crate) mod walker;

pub use anomalies::{Error, Warning};
pub(crate) use instruction::{Instruction, InstructionKind};
pub(crate) use span::{Location, Span, Spanned, Spanning};
pub(crate) use token::{Identifier, Token};
use tokenizer::Word;
pub(crate) use traits::{Stockpile, UnzipCollect};

/// Assembles an Uxn binary from a string representing an Uxntal program.
///
/// - In case the program is valid, returns an `Ok((Vec<u8>, Vec<Warning>))` &mdash; the binary
///   represented as a sequence of bytes in a `Vec`, along with any [`Warning`]s that have been
///   reported during the assembly.
/// - In case the program is invalid, i.e. it contains errors, returns an
///   `Err((Vec<Error>, Vec<Warning>))`, which contains all [`Error`]s in the program, along with
///   any [`Warning`]s that may have also been generated. The `Vec` containing the errors is always
///   non-empty.
///
/// # Example
///
/// ```rust
/// let (binary, _) = ruxnasm::assemble(b"|0100 #02 #03 ADD").unwrap();
///
/// assert_eq!(binary, [0x01, 0x02, 0x01, 0x03, 0x18]);
/// ```
pub fn assemble(source: &[u8]) -> Result<(Vec<u8>, Vec<Warning>), (Vec<Error>, Vec<Warning>)> {
    let mut warnings = Vec::new();

    let words = scanner::Scanner::new(source)
        .unzip_collect()
        .stockpile(&mut warnings)
        .map_err(|errors| (errors, warnings.clone()))?;

    let mut walker = walker::Walker::new();
    let words: Vec<&Word> = words.iter().collect();
    let mut stack: Vec<Vec<&Word>> = vec![words];
    let mut chain: Vec<(Vec<u8>, Span)> = Vec::new();

    while let Some(top) = stack.pop() {
        match walker.walk(&top) {
            Some((macro_words, macro_name, invoke_span, previous_words)) => {
                stack.push(previous_words);
                stack.push(macro_words);
                if let Some(position) = chain.iter().position(|(n, _)| *n == macro_name) {
                    let mut actual_chain = vec![(macro_name.clone(), invoke_span)];
                    actual_chain.extend(chain.iter().skip(position + 1).cloned());
                    return Err((
                        vec![Error::RecursiveMacro {
                            chain: actual_chain
                                .into_iter()
                                .map(|(macro_name, macro_span)| {
                                    (
                                        String::from_utf8_lossy(&macro_name).into_owned(),
                                        macro_span.into(),
                                    )
                                })
                                .collect(),
                            span: chain[position].1.into(),
                        }],
                        warnings,
                    ));
                } else {
                    chain.push((macro_name, invoke_span));
                }
            }
            None => {
                chain.pop();
            }
        }
    }

    let (statements, definitions) = match walker.finalize() {
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
