pub mod argument_parser;
pub mod reader;
pub mod reporter;
pub mod writer;

fn main() {
    match argument_parser::parse_arguments() {
        Ok(arguments) => match reader::read(arguments.input_file_path()) {
            Ok(input_file_contents) => {
                println!("{}", input_file_contents);
            }
            Err(error) => {
                let reporter = reporter::VoidReporter::new();
                reporter.emit(error.into());
            }
        },
        Err(error) => {
            let reporter = reporter::VoidReporter::new();
            reporter.emit(error.into());
        }
    }
}
