pub use error::Error;
use std::fs;
use std::path::Path;

mod error;

pub fn read(path: &Path) -> Result<String, Error> {
    fs::read_to_string(path).map_err(|io_error| Error::CouldNotReadFile {
        file_path: path.to_path_buf(),
        io_error,
    })
}
