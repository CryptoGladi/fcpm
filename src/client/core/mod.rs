//! Trait for opening package managers.

pub mod index;
pub mod lockfile;
pub mod options;

use crate::error::Error;
use index::Index;
use lockfile::LockFile;
use options::OpenOptions;
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur when opening a package manager.
#[derive(Debug, Error)]
pub enum OpenError {
    /// The storage directory was not found.
    #[error("Storage not found: `{0}`")]
    StorageNotFound(PathBuf),

    /// The index file was not found.
    #[error("Index not found: `{0}`")]
    IndexNotFound(PathBuf),

    /// A filesystem I/O error occurred.
    #[error("Filesystem: `{0}`")]
    IO(#[from] std::io::Error),

    /// An error occurred in the index.
    #[error("Error in index: `{0}`")]
    Index(#[from] index::IndexError),

    /// A lockfile error occurred.
    #[error("Lockfile error: `{0}`")]
    LockFile(#[from] lockfile::LockFileError),
}

pub fn check_exists_files(path: impl AsRef<Path>, options: &OpenOptions) -> Result<(), OpenError> {
    #[cfg(feature = "logging")]
    log::debug!("Run check exists files");

    let path_buf = path.as_ref().to_path_buf();

    if !fs::metadata(&path_buf)?.is_dir() {
        return Err(OpenError::StorageNotFound(path_buf));
    }

    let index_path = path_buf.join(&options.index_name);
    if !fs::metadata(&index_path)?.is_file() {
        return Err(OpenError::IndexNotFound(index_path));
    }

    Ok(())
}

/// A trait for types that can open a package manager at a given path.
///
/// This trait provides methods to initialize and access components of a package manager,
/// such as the index and lockfile, with configurable options.
///
/// # Type Parameters
///
/// * `P` - The type of package managed by the index, must implement [`Package`].
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use crate::fcpm::PackageManagerOpen;
/// # use fcpm::example::client::PackageManagerOpenTest as PackageManager;
///
/// let pm = PackageManager::open("/path/to/pm")?;
/// let index = pm.get_index();
/// # Ok::<(), fcpm::error::Error>(())
/// ```
pub trait PackageManagerCore
where
    Self: Sized,
{
    type Package: crate::package::Package;

    /// Opens a package manager at the specified path with default options.
    ///
    /// # Parameters
    /// * `path` - The path to the package manager directory.
    ///
    /// # Returns
    /// Returns a `Result` containing the opened package manager or an [`enum@Error`].
    fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        #[cfg(feature = "logging")]
        log::debug!("Open package manager in path: {}", path.as_ref().display());

        Self::open_with_options(path, OpenOptions::default())
    }

    /// Opens a package manager at the specified path with custom options.
    ///
    /// # Parameters
    /// * `path` - The path to the package manager directory.
    /// * `options` - Configuration options for opening the package manager.
    ///
    /// # Returns
    /// Returns a `Result` containing the opened package manager or an [`enum@Error`].
    fn open_with_options(path: impl AsRef<Path>, options: OpenOptions) -> Result<Self, Error>;

    /// Returns a reference to the lockfile.
    /// The lockfile prevents concurrent access to the package manager.
    ///
    /// # Returns
    /// A reference to the [`LockFile`].
    fn get_lockfile(&self) -> &LockFile;

    /// Returns a reference to the index.
    /// The index manages the packages in the package manager.
    ///
    /// # Returns
    /// A reference to the [`Index<P>`].
    fn get_index(&self) -> &Index<Self::Package>;

    /// Returns the options used to open the package manager.
    ///
    /// # Returns
    /// A [`Cow`] containing the [`OpenOptions`].
    fn get_options(&self) -> Cow<'_, OpenOptions>;

    /// Returns the path to the package manager directory.
    ///
    /// # Returns
    /// A [`Cow`] containing the path as a [`Path`].
    fn path(&self) -> Cow<'_, Path>;
}
