use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    CouldNotWriteFile {
        file_path: PathBuf,
        io_error: io::Error,
    },
}
