mod anomalies;
mod instruction;
pub mod scanner;
mod span;
mod token;
pub mod tokenizer;

pub use anomalies::{Error, Warning};
pub use instruction::{Instruction, InstructionKind};
pub use span::{Location, Span, Spanned, Spanning};
pub use token::{Identifier, Token};

pub fn assemble(
    input_file_contents: &str,
) -> Result<(Vec<u8>, Vec<Warning>), (Vec<Error>, Vec<Warning>)> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let mut words = match scanner::scan(input_file_contents) {
        Ok((words, new_warnings)) => {
            warnings.extend(new_warnings.iter().cloned().map(Warning::from));
            words
        }
        Err(error) => {
            errors.push(error.into());
            return Err((errors, warnings));
        }
    };

    for word in &mut words {
        match word.get_token() {
            Ok((token, new_warnings)) => {
                warnings.extend(new_warnings.iter().cloned().map(Warning::from));
                println!("{:?}", token.node);
            }
            Err((new_errors, new_warnings)) => {
                errors.extend(new_errors.iter().cloned().map(Error::from));
                warnings.extend(new_warnings.iter().cloned().map(Warning::from));
            }
        }
    }

    Err((errors, warnings))
}
