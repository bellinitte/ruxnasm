use super::diagnostic::{Label, LabelStyle};
use super::{FileDiagnostic, VoidDiagnostic};
use crate::{argument_parser, reader, writer};
use ruxnasm::{scanner, tokenizer};

impl From<crate::InternalAssemblerError> for VoidDiagnostic {
    fn from(error: crate::InternalAssemblerError) -> Self {
        VoidDiagnostic::bug()
            .with_message(format!("internal assembler error: {}", error.message))
            .with_note("The assembler unexpectedly panicked. This is a bug.")
            .with_note(
                "We would appreciate a bug report: https://github.com/karolbelina/ruxnasm/issues",
            )
    }
}

impl From<argument_parser::Error> for VoidDiagnostic {
    fn from(error: argument_parser::Error) -> Self {
        match error {
            argument_parser::Error::NoInputProvided => {
                VoidDiagnostic::error().with_message("no input filename given")
            }
            argument_parser::Error::NoOutputProvided => {
                VoidDiagnostic::error().with_message("no output filename given")
            }
            argument_parser::Error::UnexpectedArgument { argument } => {
                VoidDiagnostic::error().with_message(format!("unexpected argument: '{}'", argument))
            }
            argument_parser::Error::UnrecognizedOption { option } => {
                VoidDiagnostic::error().with_message(format!("unrecognized option: '{}'", option))
            }
        }
    }
}

impl From<reader::Error> for VoidDiagnostic {
    fn from(error: reader::Error) -> Self {
        match error {
            reader::Error::CouldNotReadFile {
                file_path,
                io_error,
            } => VoidDiagnostic::error().with_message(format!(
                "couldn't read {}: {}",
                file_path.to_string_lossy(),
                io_error
            )),
        }
    }
}

impl From<writer::Error> for VoidDiagnostic {
    fn from(error: writer::Error) -> Self {
        match error {
            writer::Error::CouldNotWriteFile {
                file_path,
                io_error,
            } => VoidDiagnostic::error().with_message(format!(
                "couldn't write {}: {}",
                file_path.to_string_lossy(),
                io_error
            )),
        }
    }
}

impl From<ruxnasm::Error> for FileDiagnostic {
    fn from(error: ruxnasm::Error) -> Self {
        match error {
            ruxnasm::Error::Scanner(error) => error.into(),
            ruxnasm::Error::Tokenizer(error) => error.into(),
        }
    }
}

impl From<ruxnasm::Warning> for FileDiagnostic {
    fn from(warning: ruxnasm::Warning) -> Self {
        match warning {
            ruxnasm::Warning::Scanner(warning) => warning.into(),
            ruxnasm::Warning::Tokenizer(warning) => warning.into(),
        }
    }
}

impl From<scanner::Warning> for FileDiagnostic {
    fn from(warning: scanner::Warning) -> Self {
        match warning {
            scanner::Warning::TokenTrimmed { span } => FileDiagnostic::warning()
                .with_message(format!(
                    "token has been cut off, as it's longer than 64 characters"
                ))
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
        }
    }
}

impl From<tokenizer::Warning> for FileDiagnostic {
    fn from(warning: tokenizer::Warning) -> Self {
        match warning {
            tokenizer::Warning::InstructionModeDefinedMoreThanOnce {
                instruction_mode,
                instruction,
                span,
                other_span,
            } => FileDiagnostic::warning()
                .with_message(format!(
                    "instruction mode `{}` is defined multiple times for instruction `{}`",
                    instruction_mode, instruction
                ))
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: format!("mode `{}` redefined here", instruction_mode),
                })
                .with_label(Label {
                    style: LabelStyle::Secondary,
                    span: other_span,
                    message: format!("previous definition of mode `{}` here", instruction_mode),
                }),
        }
    }
}

impl From<scanner::Error> for FileDiagnostic {
    fn from(error: scanner::Error) -> Self {
        match error {
            scanner::Error::NoMatchingClosingParenthesis { span } => FileDiagnostic::error()
                .with_message("no matching closing parenthesis found for an opening parenthesis")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            scanner::Error::NoMatchingOpeningParenthesis { span } => FileDiagnostic::error()
                .with_message("no matching opening parenthesis found for a closing parenthesis")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
        }
    }
}

impl From<tokenizer::Error> for FileDiagnostic {
    fn from(error: tokenizer::Error) -> Self {
        match error {
            tokenizer::Error::MacroNameExpected { span } => FileDiagnostic::error()
                .with_message("expected a macro name")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            tokenizer::Error::LabelExpected { span } => FileDiagnostic::error()
                .with_message("expected an label name")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            tokenizer::Error::IdentifierExpected { span } => FileDiagnostic::error()
                .with_message("expected an identifier")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            tokenizer::Error::HexNumberExpected { span } => FileDiagnostic::error()
                .with_message("expected a hexadecimal number")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            tokenizer::Error::HexDigitInvalid {
                digit,
                number,
                span,
            } => FileDiagnostic::error()
                .with_message(format!(
                    "invalid digit `{}` in a hexadecimal number `{}`",
                    digit, number
                ))
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            tokenizer::Error::HexNumberUnevenLength {
                length,
                number,
                span,
            } => FileDiagnostic::error()
                .with_message(format!(
                    "hexadecimal number `{}` has an uneven length of {}",
                    number, length
                ))
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                })
                .with_note("help: pad the number with zeros"),
            tokenizer::Error::HexNumberTooLong {
                length,
                number,
                span,
            } => FileDiagnostic::error()
                .with_message(format!(
                    "hexadecimal number `{}` of length {} is too long",
                    number, length
                ))
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            tokenizer::Error::MacroCannotBeAHexNumber { number, span } => FileDiagnostic::error()
                .with_message(format!(
                    "`{}` cannot be used as a macro name, as it is a valid hexadecimal number",
                    number
                ))
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            tokenizer::Error::MacroCannotBeAnInstruction { instruction, span } => {
                FileDiagnostic::error()
                    .with_message(format!(
                        "`{}` cannot be used as a macro name, as it is a valid instruction",
                        instruction
                    ))
                    .with_label(Label {
                        style: LabelStyle::Primary,
                        span,
                        message: String::new(),
                    })
            }
        }
    }
}
