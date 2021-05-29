use std::io;
use std::path::PathBuf;

pub enum Error {
    CouldNotReadFile {
        file_path: PathBuf,
        io_error: io::Error,
    },
}
