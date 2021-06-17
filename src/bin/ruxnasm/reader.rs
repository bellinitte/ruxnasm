use std::path::{Path, PathBuf};
use std::{fs, io};

pub enum Error {
    CouldNotReadFile {
        file_path: PathBuf,
        io_error: io::Error,
    },
}

pub fn read(path: &Path) -> Result<Vec<u8>, Error> {
    fs::read(path).map_err(|io_error| Error::CouldNotReadFile {
        file_path: path.to_path_buf(),
        io_error,
    })
}
