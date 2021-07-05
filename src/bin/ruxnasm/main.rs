use std::panic::set_hook;
use std::process::exit;

pub mod argument_parser;
pub mod reader;
pub mod reporter;
pub mod writer;

struct InternalAssemblerError {
    message: String,
}

fn try_main() -> Result<(), ()> {
    match argument_parser::parse_arguments() {
        Ok(arguments) => match reader::read(arguments.input_file_path()) {
            Ok(input_file_contents) => {
                let reporter = reporter::VoidReporter::new()
                    .promote(arguments.input_file_path(), &input_file_contents);
                match ruxnasm::assemble(&input_file_contents) {
                    Ok((binary, warnings)) => {
                        for warning in warnings {
                            reporter.emit(warning.into());
                        }
                        match writer::write(arguments.output_file_path(), &binary) {
                            Ok(()) => Ok(()),
                            Err(error) => {
                                let reporter = reporter.demote();
                                reporter.emit(error.into());
                                Err(())
                            }
                        }
                    }
                    Err((errors, warnings)) => {
                        for error in errors {
                            reporter.emit(error.into());
                        }
                        for warning in warnings {
                            reporter.emit(warning.into());
                        }
                        Err(())
                    }
                }
            }
            Err(error) => {
                let reporter = reporter::VoidReporter::new();
                reporter.emit(error.into());
                Err(())
            }
        },
        Err(error) => {
            let reporter = reporter::VoidReporter::new();
            reporter.emit(error.into());
            Err(())
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

        exit(1);
    }));

    let exit_code = try_main();

    exit(exit_code.is_err() as i32);
}
