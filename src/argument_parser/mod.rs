use std::{env, path::PathBuf, process::exit};
pub use error::Error;

mod error;

const HELP_MESSAGE: &'static str = r#"Usage: ruxnasm [OPTIONS] INPUT OUTPUT

Options:
    -h, --help          Display this message
    -V, --version       Print version info and exit
"#;
const VERSION_MESSAGE: &'static str = concat!("ruxnasm ", env!("CARGO_PKG_VERSION"));

#[derive(Debug)]
pub struct Arguments {
    input: PathBuf,
    output: PathBuf,
}

pub fn parse_arguments() -> Result<Arguments, Error> {
    if env::args().len() == 1 {
        exit_with_help_message();
    }

    let mut args = env::args();
    args.next();
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;

    for arg in args {
        if arg.starts_with("--") {
            match &arg[2..] {
                "help" => {
                    exit_with_help_message();
                },
                "version" => {
                    exit_with_version_message();
                }
                option => return Err(Error::UnrecognizedOption { option: option.to_owned() })
            }
        } else if arg.starts_with("-") {
            for ch in arg[1..].chars() {
                match ch {
                    'h' => {
                        exit_with_help_message();
                    },
                    'V' => {
                        exit_with_version_message();
                    }
                    option => return Err(Error::UnrecognizedOption { option: option.to_string() })
                }
            }
        } else {
            if input.is_none() {
                input = Some(arg.into());
            } else if output.is_none() {
                output = Some(arg.into());
            } else {
                return Err(Error::UnexpectedArgument { argument: arg.to_owned() });
            }
        }
    }

    match (input, output) {
        (Some(input), Some(output)) => Ok(Arguments { input, output }),
        (None, _) => Err(Error::NoInputProvided),
        (_, None) => Err(Error::NoOutputProvided),
    }
}

fn exit_with_help_message() {
    println!("{}", HELP_MESSAGE);
    exit(0);
}

fn exit_with_version_message() {
    println!("{}", VERSION_MESSAGE);
    exit(0);
}
