use crate::argument_parser;
use super::VoidDiagnostic;

impl From<argument_parser::Error> for VoidDiagnostic {
    fn from(error: argument_parser::Error) -> Self {
        match error {
            argument_parser::Error::NoInputProvided => VoidDiagnostic::error()
                .with_message("no input filename given"),
            argument_parser::Error::NoOutputProvided => VoidDiagnostic::error()
                .with_message("no output filename given"),
            argument_parser::Error::UnexpectedArgument { argument } => VoidDiagnostic::error()
                .with_message(format!("unexpected argument: '{}'", argument)),
            argument_parser::Error::UnrecognizedOption { option } => VoidDiagnostic::error()
                .with_message(format!("unrecognized option: '{}'", option))
        }
    }
}
