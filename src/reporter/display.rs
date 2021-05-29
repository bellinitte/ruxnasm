use super::VoidDiagnostic;
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
