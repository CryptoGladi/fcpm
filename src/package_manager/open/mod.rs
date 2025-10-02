pub(crate) mod index;
pub(crate) mod manifest;

use crate::error::Error;
use core::error;
use index::Index;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const INDEX_NAME: &str = "index.sqlite";
const MANIFEST_NAME: &str = "manifest.toml";

#[derive(Debug, Clone)]
pub struct OpenOptions {
    create_if_not_exists: bool,
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self {
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

    #[error("Manifest not found: `{0}`")]
    ManifestNotFound(PathBuf),

    #[error("Filesystem: `{0}`")]
    IO(#[from] std::io::Error),

    #[error("Error in index sqlite: `{0}`")]
    Index(#[from] rusqlite::Error),
}

fn check_exists_files(path: impl AsRef<Path>) -> Result<(), OpenError> {
    let path_buf = path.as_ref().to_path_buf();

    if !fs::metadata(&path_buf)?.is_dir() {
        return Err(OpenError::StorageNotFound(path_buf));
    }

    let index_path = path_buf.join(INDEX_NAME);
    if !fs::metadata(&index_path)?.is_file() {
        return Err(OpenError::IndexNotFound(index_path));
    }

    let manifest_path = path_buf.join(MANIFEST_NAME);
    if !fs::metadata(&manifest_path)?.is_file() {
        return Err(OpenError::ManifestNotFound(manifest_path));
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

        check_exists_files(&path_buf)?;
        let index = Index::open(&path_buf, options.into())?;

        todo!()
    }

    fn open_with_options(path: impl AsRef<Path>, options: OpenOptions) -> Result<Self, Error>;
}
