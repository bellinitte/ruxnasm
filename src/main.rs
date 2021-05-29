pub mod argument_parser;
pub mod reader;
pub mod reporter;
pub mod writer;

fn main() {
    match argument_parser::parse_arguments() {
        Ok(arguments) => {
            println!("{:?}", arguments);
        }
        Err(error) => {
            let reporter = reporter::VoidReporter::new();
            reporter.emit(error.into());
        }
    }
}
