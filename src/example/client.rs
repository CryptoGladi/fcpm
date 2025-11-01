use crate::client::core::{OpenError, check_exists_files};
use crate::error::Error;
use crate::{Index, LockFile, OpenOptions, PackageManagerCore, example::package::PackageExample};
use std::borrow::Cow;
use std::path::{Path, PathBuf};

pub struct PackageManagerOpenTest {
    lockfile: LockFile,
    index: Index<PackageExample>,
    options: OpenOptions,
    path: PathBuf,
}

impl PackageManagerCore for PackageManagerOpenTest {
    type Package = PackageExample;

    fn open_with_options(path: impl AsRef<Path>, options: OpenOptions) -> Result<Self, Error> {
        let path_buf = path.as_ref().to_path_buf();

        if !options.create_if_not_exists {
            check_exists_files(&path_buf, &options)?;
        }

        let lockfile =
            LockFile::new(path_buf.join(&options.lockfile_name)).map_err(OpenError::LockFile)?;

        let index = Index::open(path_buf.join(&options.index_name), options.clone().into())
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{TempDir, tempdir};

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
