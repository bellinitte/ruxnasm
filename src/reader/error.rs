use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    CouldNotReadFile {
        file_path: PathBuf,
        io_error: io::Error,
    },
}
