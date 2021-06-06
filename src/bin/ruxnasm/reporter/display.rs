use super::diagnostic::{Label, LabelStyle};
use super::{FileDiagnostic, VoidDiagnostic};
use crate::{argument_parser, reader, writer};

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
            ruxnasm::Error::NoMatchingClosingParenthesis { span } => FileDiagnostic::error()
                .with_message("no matching closing parenthesis found for an opening parenthesis")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            ruxnasm::Error::NoMatchingOpeningParenthesis { span } => FileDiagnostic::error()
                .with_message("no matching opening parenthesis found for a closing parenthesis")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),

            ruxnasm::Error::MacroNameExpected { span } => FileDiagnostic::error()
                .with_message("expected a macro name")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            ruxnasm::Error::LabelExpected { span } => FileDiagnostic::error()
                .with_message("expected an label name")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            ruxnasm::Error::SublabelExpected { span } => FileDiagnostic::error()
                .with_message("expected an sublabel name")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            ruxnasm::Error::SlashInLabelOrSublabel { span } => FileDiagnostic::error()
                .with_message("label and sublabel names can't include the '/' character")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            ruxnasm::Error::MoreThanOneSlashInIdentifier { span } => FileDiagnostic::error()
                .with_message("identifiers can't have more than one '/' character")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            ruxnasm::Error::MoreThanOneByteFound { bytes, span } => FileDiagnostic::error()
                .with_message("found more than one byte after a raw character rune")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: format!("found bytes: {:x?}", bytes),
                }),
            ruxnasm::Error::AmpersandAtTheStartOfLabel { span } => FileDiagnostic::error()
                .with_message("label names can't have '&' as their first character")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            ruxnasm::Error::IdentifierExpected { span } => FileDiagnostic::error()
                .with_message("expected an identifier")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            ruxnasm::Error::HexNumberExpected { span } => FileDiagnostic::error()
                .with_message("expected a hexadecimal number")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            ruxnasm::Error::HexNumberOrCharacterExpected { span } => FileDiagnostic::error()
                .with_message("expected a hexadecimal number or a character")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            ruxnasm::Error::CharacterExpected { span } => FileDiagnostic::error()
                .with_message("expected a character")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            ruxnasm::Error::HexDigitInvalid {
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
            ruxnasm::Error::HexNumberUnevenLength {
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
            ruxnasm::Error::HexNumberTooLong {
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
            ruxnasm::Error::MacroCannotBeAHexNumber { number, span } => FileDiagnostic::error()
                .with_message(format!(
                    "`{}` cannot be used as a macro name, as it is a valid hexadecimal number",
                    number
                ))
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            ruxnasm::Error::MacroCannotBeAnInstruction { instruction, span } => {
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
            ruxnasm::Error::MacroUndefined { name, span } => FileDiagnostic::error()
                .with_message(format!("macro `{}` is not defined", name))
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            ruxnasm::Error::MacroDefinedMoreThanOnce {
                name,
                span,
                other_span,
            } => FileDiagnostic::error()
                .with_message(format!("macro `{}` is defined multiple times", name))
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: format!("macro `{}` redefined here", name),
                })
                .with_label(Label {
                    style: LabelStyle::Secondary,
                    span: other_span,
                    message: format!("previous definition of macro `{}` here", name),
                }),
            ruxnasm::Error::LabelDefinedMoreThanOnce {
                name,
                span,
                other_span,
            } => FileDiagnostic::error()
                .with_message(format!("label `{}` is defined multiple times", name))
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: format!("label `{}` redefined here", name),
                })
                .with_label(Label {
                    style: LabelStyle::Secondary,
                    span: other_span,
                    message: format!("previous definition of label `{}` here", name),
                }),
            ruxnasm::Error::OpeningBraceNotAfterMacroDefinition { span } => FileDiagnostic::error()
                .with_message("found an opening brace that is not a part of a macro definition")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            ruxnasm::Error::NoMatchingOpeningBrace { span } => FileDiagnostic::error()
                .with_message("no matching opening brace found for a closing brace")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            ruxnasm::Error::NoMatchingClosingBrace { span } => FileDiagnostic::error()
                .with_message("no matching closing brace found for an opening brace")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            ruxnasm::Error::SublabelDefinedWithoutScope { name, span } => FileDiagnostic::error()
                .with_message(format!(
                    "sublabel `{}` was defined without a previously defined label",
                    name
                ))
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            ruxnasm::Error::NoMatchingOpeningBracket { span } => FileDiagnostic::error()
                .with_message("no matching opening bracket found for a closing bracket")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            ruxnasm::Error::NoMatchingClosingBracket { span } => FileDiagnostic::error()
                .with_message("no matching closing bracket found for an opening bracket")
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
        }
    }
}

impl From<ruxnasm::Warning> for FileDiagnostic {
    fn from(warning: ruxnasm::Warning) -> Self {
        match warning {
            ruxnasm::Warning::TokenTrimmed { span } => FileDiagnostic::warning()
                .with_message(format!(
                    "token has been cut off, as it's longer than 64 characters"
                ))
                .with_label(Label {
                    style: LabelStyle::Primary,
                    span,
                    message: String::new(),
                }),
            ruxnasm::Warning::InstructionModeDefinedMoreThanOnce {
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
