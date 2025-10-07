//! Core trait

pub mod index;
pub mod lockfile;

use crate::error::Error;
use crate::package::Package;
use index::Index;
use lockfile::LockFile;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenOptions {
    index_name: String,
    lockfile_name: String,
    repository_name: String,
    repository_lock_name: String,
    create_if_not_exists: bool,
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self {
            index_name: "index.sqlite".to_string(),
            lockfile_name: "fcpm.lock".to_string(),
            repository_name: "repo.json".to_string(),
            repository_lock_name: "repo.lock".to_string(),
            create_if_not_exists: true,
        }
    }
}

impl From<OpenOptions> for rusqlite::OpenFlags {
    fn from(value: OpenOptions) -> Self {
        use rusqlite::OpenFlags;

        let mut open_flags = OpenFlags::SQLITE_OPEN_NO_MUTEX
            | OpenFlags::SQLITE_OPEN_URI
            | OpenFlags::SQLITE_OPEN_READ_WRITE;

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

pub trait PackageManagerOpen<Metadata, P>
where
    Self: Sized,
    Metadata: Serialize + DeserializeOwned + Clone,
    P: Package<Metadata>,
{
    fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        #[cfg(feature = "logging")]
        log::debug!("Open package manager in path: {}", path.as_ref().display());

        Self::open_with_options(path, OpenOptions::default())
    }

    fn open_with_options(path: impl AsRef<Path>, options: OpenOptions) -> Result<Self, Error>;

    fn get_lockfile(&self) -> &LockFile;

    fn get_index(&self) -> &Index<Metadata, P>;

    fn get_options(&self) -> Cow<'_, OpenOptions>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::tests::PackageTest;
    use tempfile::{TempDir, tempdir};

    pub(crate) struct PackageManagerOpenTest {
        lockfile: LockFile,
        index: Index<(), PackageTest>,
        options: OpenOptions,
    }

    impl PackageManagerOpen<(), PackageTest> for PackageManagerOpenTest {
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
            })
        }

        fn get_lockfile(&self) -> &LockFile {
            &self.lockfile
        }

        fn get_index(&self) -> &Index<(), PackageTest> {
            &self.index
        }

        fn get_options(&self) -> Cow<'_, OpenOptions> {
            Cow::Borrowed(&self.options)
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
}
