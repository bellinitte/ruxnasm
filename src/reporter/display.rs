use super::diagnostic::{Label, LabelStyle};
use super::{FileDiagnostic, VoidDiagnostic};
use crate::{argument_parser, reader, writer};
use ruxnasm::tokenizer;

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
            ruxnasm::Error::Tokenizer(error) => error.into(),
        }
    }
}

impl From<tokenizer::Error> for FileDiagnostic {
    fn from(error: tokenizer::Error) -> Self {
        match error {
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
            tokenizer::Error::HexDigitInvalid { digit, span } => FileDiagnostic::error()
                .with_message(format!("invalid hexadecimal digit `{}`", digit))
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            tokenizer::Error::HexNumberUnevenLength { span } => FileDiagnostic::error()
                .with_message(format!("uneven number of digits in a hexadecimal number"))
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                })
                .with_note("help: pad the number with zeros"),
            tokenizer::Error::HexNumberTooLarge { length: _, span } => FileDiagnostic::error()
                .with_message(format!("hexadecimal number is too large"))
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            tokenizer::Error::CharacterExpected { span } => FileDiagnostic::error()
                .with_message("expected a character")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            tokenizer::Error::InstructionInvalid { instruction, span } => FileDiagnostic::error()
                .with_message(format!("invalid instruction '{}'", instruction))
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            tokenizer::Error::InstructionModeInvalid {
                instruction_mode,
                span,
            } => FileDiagnostic::error()
                .with_message(format!("invalid instruction mode '{}'", instruction_mode))
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            tokenizer::Error::InstructionModeDefinedMoreThanOnce {
                instruction_mode,
                span,
                other_span,
            } => FileDiagnostic::error()
                .with_message(format!(
                    "instruction mode '{}' is defined multiple times for an instruction",
                    instruction_mode
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
            tokenizer::Error::IdentifierCannotBeAHexNumber { span } => FileDiagnostic::error()
                .with_message("identifiers cannot be valid hexadecimal numbers")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            tokenizer::Error::IdentifierCannotBeAnInstructon { span } => FileDiagnostic::error()
                .with_message("identifiers cannot be valid instructions")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
        }
    }
}
