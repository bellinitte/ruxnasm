use std::path::{Path, PathBuf};
use std::{
    fs,
    io::{self, Write},
};

pub enum Error {
    CouldNotWriteFile {
        file_path: PathBuf,
        io_error: io::Error,
    },
}

pub fn write(path: &Path, binary: &[u8]) -> Result<(), Error> {
    let mut file = fs::File::create(path).map_err(|io_error| Error::CouldNotWriteFile {
        file_path: path.to_path_buf(),
        io_error,
    })?;
    file.write_all(binary)
        .map_err(|io_error| Error::CouldNotWriteFile {
            file_path: path.to_path_buf(),
            io_error,
        })?;
    Ok(())
}
