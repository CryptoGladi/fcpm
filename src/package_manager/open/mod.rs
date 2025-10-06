pub mod index;
pub mod lockfile;

use crate::error::Error;
use index::Index;
use lockfile::LockFile;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct OpenOptions {
    index_name: String,
    lockfile_name: String,
    create_if_not_exists: bool,
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self {
            index_name: "index.sqlite".to_string(),
            lockfile_name: "fcpm.lock".to_string(),
            create_if_not_exists: true,
        }
    }
}

impl From<OpenOptions> for rusqlite::OpenFlags {
    fn from(value: OpenOptions) -> Self {
        let mut open_flags = rusqlite::OpenFlags::empty();

        if value.create_if_not_exists {
            open_flags |= rusqlite::OpenFlags::SQLITE_OPEN_CREATE;
        }

        open_flags
    }
}

#[derive(Debug, Error)]
pub enum OpenError {
    #[error("Storage not found: `{0}`")]
    StorageNotFound(PathBuf),

    #[error("Index not found: `{0}`")]
    IndexNotFound(PathBuf),

    #[error("Filesystem: `{0}`")]
    IO(#[from] std::io::Error),

    #[error("Error in index: `{0}`")]
    Index(#[from] index::IndexError),

    #[error("Lockfile error: `{0}`")]
    LockFile(#[from] lockfile::LockFileError),
}

fn check_exists_files(path: impl AsRef<Path>, config: &OpenOptions) -> Result<(), OpenError> {
    let path_buf = path.as_ref().to_path_buf();

    if !fs::metadata(&path_buf)?.is_dir() {
        return Err(OpenError::StorageNotFound(path_buf));
    }

    let index_path = path_buf.join(&config.index_name);
    if !fs::metadata(&index_path)?.is_file() {
        return Err(OpenError::IndexNotFound(index_path));
    }

    Ok(())
}

pub trait PackageManagerOpen
where
    Self: Sized,
{
    fn open(path: impl AsRef<Path>, options: OpenOptions) -> Result<Self, OpenError> {
        let path_buf = path.as_ref().to_path_buf();

        #[cfg(feature = "logging")]
        log::debug!("Open from path: {}", path_buf.display());

        let lock = LockFile::new(path_buf.join(&options.lockfile_name))?;

        check_exists_files(&path_buf, &options)?;
        //let index: Index<, _> = Index::open(path_buf.join(&options.index_name), options.into())?;

        todo!()
    }

    fn open_with_options(path: impl AsRef<Path>, options: OpenOptions) -> Result<Self, Error>;
}

#[cfg(test)]
mod tests {}
