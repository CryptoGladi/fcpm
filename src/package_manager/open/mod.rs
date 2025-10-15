//! Trait for opening package managers.

pub mod index;
pub mod lockfile;
pub mod options;

use crate::error::Error;
use crate::package::Package;
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
/// * `P` - The type of package managed by the index, must implement [`Package`].
///
/// # Examples
///
/// ```no_run
/// use fcpm::package_manager::open::{PackageManagerOpen, Options};
/// use std::path::Path;
///
/// let pm = PackageManagerOpenTest::open("/path/to/pm")?;
/// let index = pm.get_index();
/// ```
pub trait PackageManagerOpen<P>
where
    Self: Sized,
    P: Package,
{
    /// Opens a package manager at the specified path with default options.
    ///
    /// # Parameters
    /// * `path` - The path to the package manager directory.
    ///
    /// # Returns
    /// Returns a `Result` containing the opened package manager or an [`Error`].
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
    /// Returns a `Result` containing the opened package manager or an [`Error`].
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
    fn get_index(&self) -> &Index<P>;

    /// Returns the options used to open the package manager.
    ///
    /// # Returns
    /// A [`Cow`] containing the [`Options`].
    fn get_options(&self) -> Cow<'_, OpenOptions>;

    /// Returns the path to the package manager directory.
    ///
    /// # Returns
    /// A [`Cow`] containing the path as a [`Path`].
    fn path(&self) -> Cow<'_, Path>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::example::package::PackageExample;
    use tempfile::{TempDir, tempdir};

    pub(crate) struct PackageManagerOpenTest {
        lockfile: LockFile,
        index: Index<PackageExample>,
        options: OpenOptions,
        path: PathBuf,
    }

    impl PackageManagerOpen<PackageExample> for PackageManagerOpenTest {
        fn open_with_options(path: impl AsRef<Path>, options: OpenOptions) -> Result<Self, Error> {
            let path_buf = path.as_ref().to_path_buf();

            if !options.create_if_not_exists {
                check_exists_files(&path_buf, &options)?;
            }

            let lockfile = LockFile::new(&path_buf.join(&options.lockfile_name))
                .map_err(OpenError::LockFile)?;
            let index = Index::open(&path_buf.join(&options.index_name), options.clone().into())
                .map_err(OpenError::Index)?;

            Ok(Self {
                index,
                lockfile,
                options,
                path: path_buf,
            })
        }

        fn get_lockfile(&self) -> &LockFile {
            &self.lockfile
        }

        fn get_index(&self) -> &Index<PackageExample> {
            &self.index
        }

        fn get_options(&self) -> Cow<'_, OpenOptions> {
            Cow::Borrowed(&self.options)
        }

        fn path(&self) -> Cow<'_, Path> {
            Cow::Borrowed(&self.path)
        }
    }

    impl PackageManagerOpenTest {
        pub(crate) fn test_create() -> (TempDir, Self) {
            let tempdir = tempdir().unwrap();
            let package_manager = PackageManagerOpenTest::open(&tempdir).unwrap();

            (tempdir, package_manager)
        }
    }

    #[test_log::test]
    fn open() {
        let tempdir = tempdir().unwrap();

        let _package_manager_open = PackageManagerOpenTest::open(tempdir).unwrap();
    }

    #[test_log::test]
    #[should_panic]
    fn open_without_create_on_open() {
        let tempdir = tempdir().unwrap();
        let options = OpenOptions {
            create_if_not_exists: false,
            ..Default::default()
        };

        let _package_manager_open =
            PackageManagerOpenTest::open_with_options(tempdir, options).unwrap();
    }

    #[test_log::test]
    fn get_lockfile() {
        let (_tempdir, package_manager) = PackageManagerOpenTest::test_create();

        let lockfile = package_manager.get_lockfile();
        assert_eq!(*lockfile, package_manager.lockfile);
    }

    #[test_log::test]
    fn get_index() {
        let (_tempdir, package_manager) = PackageManagerOpenTest::test_create();

        let index = package_manager.get_index();
        assert_eq!(*index, package_manager.index);
    }

    #[test_log::test]
    fn get_options() {
        let (_tempdir, package_manager) = PackageManagerOpenTest::test_create();

        let options = package_manager.get_options().into_owned();
        assert_eq!(options, package_manager.options);
    }

    #[test_log::test]
    fn get_path() {
        let (tempdir, package_manager) = PackageManagerOpenTest::test_create();

        assert_eq!(package_manager.path(), tempdir.path());
    }
}
