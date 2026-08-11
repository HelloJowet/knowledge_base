use crate::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn read(root: &Path, directory: &str, id: &str, extension: &str) -> Result<String, Error> {
    let path = path(root, directory, id, extension);
    fs::read_to_string(&path).map_err(|source| Error::Read { path, source })
}

pub(crate) fn path(root: &Path, directory: &str, id: &str, extension: &str) -> PathBuf {
    root.join(directory).join(Path::new(id).with_extension(extension))
}
