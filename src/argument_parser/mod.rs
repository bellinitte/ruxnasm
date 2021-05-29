pub use error::Error;
use std::{
    env,
    path::{Path, PathBuf},
    process::exit,
};

mod error;

const HELP_MESSAGE: &'static str = r#"Usage: ruxnasm [OPTIONS] INPUT OUTPUT

Options:
    -h, --help          Display this message
    -V, --version       Print version info and exit
"#;
const VERSION_MESSAGE: &'static str = concat!("ruxnasm ", env!("CARGO_PKG_VERSION"));

#[derive(Debug)]
pub struct Arguments {
    input_file_path: PathBuf,
    output_file_path: PathBuf,
}

impl Arguments {
    pub fn input_file_path(&self) -> &Path {
        &self.input_file_path
    }

    pub fn output_file_path(&self) -> &Path {
        &self.output_file_path
    }
}

pub fn parse_arguments() -> Result<Arguments, Error> {
    if env::args().len() == 1 {
        exit_with_help_message();
    }

    let mut args = env::args();
    args.next();
    let mut input_file_path: Option<PathBuf> = None;
    let mut output_file_path: Option<PathBuf> = None;

    for arg in args {
        if arg.starts_with("--") {
            match &arg[2..] {
                "help" => exit_with_help_message(),
                "version" => exit_with_version_message(),
                option => {
                    return Err(Error::UnrecognizedOption {
                        option: option.to_owned(),
                    })
                }
            }
        } else if arg.starts_with("-") {
            for ch in arg[1..].chars() {
                match ch {
                    'h' => exit_with_help_message(),
                    'V' => exit_with_version_message(),
                    option => {
                        return Err(Error::UnrecognizedOption {
                            option: option.to_string(),
                        })
                    }
                }
            }
        } else {
            if input_file_path.is_none() {
                input_file_path = Some(arg.into());
            } else if output_file_path.is_none() {
                output_file_path = Some(arg.into());
            } else {
                return Err(Error::UnexpectedArgument {
                    argument: arg.to_owned(),
                });
            }
        }
    }

    match (input_file_path, output_file_path) {
        (Some(input_file_path), Some(output_file_path)) => Ok(Arguments {
            input_file_path,
            output_file_path,
        }),
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
