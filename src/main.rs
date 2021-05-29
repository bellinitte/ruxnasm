use std::panic::set_hook;

pub mod argument_parser;
pub mod reader;
pub mod reporter;
pub mod writer;

struct InternalAssemblerError {
    message: String,
}

fn try_main() {
    match argument_parser::parse_arguments() {
        Ok(arguments) => match reader::read(arguments.input_file_path()) {
            Ok(input_file_contents) => {
                match ruxnasm::assemble(&input_file_contents) {
                    Ok(_binary) => {
                        // println!("{:?}", binary);
                    }
                    Err(errors) => {
                        let reporter = reporter::VoidReporter::new()
                            .promote(arguments.input_file_path(), &input_file_contents);
                        for error in errors {
                            reporter.emit(error.into());
                        }
                    }
                }
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

fn main() {
    set_hook(Box::new(|panic_info| {
        let reporter = reporter::VoidReporter::new();

        let error = InternalAssemblerError {
            message: panic_info.to_string(),
        };

        reporter.emit(error.into());
    }));

    try_main();
}
