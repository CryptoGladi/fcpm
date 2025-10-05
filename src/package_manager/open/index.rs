use crate::package::Package;
use rusqlite::Connection;
use serde::{Serialize, de::DeserializeOwned};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug)]
pub struct Index {
    path: PathBuf,
    db: rusqlite::Connection,
}

#[derive(Debug, Error)]
pub enum IndexError {
    #[error("SQLite error: `{0}`")]
    SQLite(#[from] rusqlite::Error),

    #[error("Json error: `{0}`")]
    Json(#[from] serde_json::Error),

    #[error("Package `{0}` not found")]
    PackageNotFound(String),
}

impl Index {
    pub fn open(
        path: impl AsRef<Path>,
        open_flags: rusqlite::OpenFlags,
    ) -> Result<Self, IndexError> {
        let path_buf = path.as_ref().to_path_buf();

        #[cfg(feature = "logging")]
        log::debug!("Open index to path: {}", path_buf.display());

        let db = Connection::open_with_flags(path, open_flags)?;
        let index = Self { path: path_buf, db };

        index.init()?;

        Ok(index)
    }

    fn init(&self) -> Result<(), IndexError> {
        #[cfg(feature = "logging")]
        log::debug!("Init database for index in path: `{}`", self.path.display());

        #[cfg(feature = "logging")]
        log::debug!("Create table `packages`");
        self.db.execute(
            r#"create table if not exists `packages` (
                    `id` integer not null primary key autoincrement,
                    `name` varchar(255) not null unique CHECK(`name` != ""),
                    `version` varchar(255) not null CHECK(`version` != ""),
                    `repository_name` varchar(255) not null CHECK(`repository_name` != ""),
                    `hashsum` varchar(256) not null CHECK(`hashsum` != ""),
                    `metadata` text null
                )"#,
            (),
        )?;

        #[cfg(feature = "logging")]
        log::debug!("Create index for table `packages`");
        self.db.execute(
            "create unique index if not exists package_name_idx on packages(name)",
            (),
        )?;

        Ok(())
    }

    pub fn add_package<Metadata>(&self, package: &impl Package<Metadata>) -> Result<(), IndexError>
    where
        Metadata: Serialize + DeserializeOwned + Clone,
    {
        #[cfg(feature = "logging")]
        log::debug!("Add package `{}`", package.name());

        // TODO is Metadata == (), skip
        let metadata = serde_json::to_string(&package.metadata())?;

        self.db.execute(
            "insert into `packages` (`name`, `version`, `repository_name`, `hashsum`, `metadata`)
                              values (?1, ?2, ?3, ?4, ?5)",
            (
                package.name(),
                package.version(),
                package.repository_name(),
                package.hashsum(),
                metadata,
            ),
        )?;

        Ok(())
    }

    pub fn delete_package(&self, package_name: &str) -> Result<(), IndexError> {
        #[cfg(feature = "logging")]
        log::debug!("Delete package by name: `{package_name}`");

        let changed = self.db.execute(
            "delete from `packages` where `name` = ?1",
            ((package_name),),
        )?;

        if changed == 0 {
            return Err(IndexError::PackageNotFound(package_name.to_string()));
        }

        Ok(())
    }

    pub fn get_package<Metadata, P>(&self, package_name: &str) -> Result<P, IndexError>
    where
        Metadata: Serialize + DeserializeOwned + Clone,
        P: Package<Metadata>,
    {
        let stmt = self.db.prepare(
            "select `version`, `repository_name`, `checksum`, `metadata` from `package` where :name",
        )?;
        //let package_iter = stmt.query_map(|row| {});
        todo!()
    }

    pub fn have_package(&self, package_name: &str) -> Result<bool, IndexError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::tests::PackageTest;
    use rusqlite::OpenFlags;
    use tempfile::{TempDir, tempdir};

    pub(crate) fn create_test_index() -> (TempDir, Index) {
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("index.sqlite");

        let index = Index::open(&path, OpenFlags::default()).unwrap();
        (tempdir, index)
    }

    #[test_log::test]
    fn open() {
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("index.sqlite");

        let index = Index::open(&path, OpenFlags::default()).unwrap();

        assert_eq!(index.path, path);
    }

    #[test_log::test]
    fn init() {
        let (_tempdir, index) = create_test_index();

        index.init().unwrap();
    }

    #[test_log::test]
    fn add_package() {
        let (_tempdir, index) = create_test_index();
        let package_test = PackageTest::default();

        index.add_package(&package_test).unwrap();
        println!("ds");
    }

    #[test_log::test]
    #[should_panic]
    fn add_package_with_same_name() {
        let (_tempdir, index) = create_test_index();

        let package_test1 = PackageTest {
            name: "supername".to_string(),
            ..Default::default()
        };
        let package_test2 = PackageTest {
            name: "supername".to_string(),
            ..Default::default()
        };

        index.add_package(&package_test1).unwrap();
        index.add_package(&package_test2).unwrap(); // Panic
    }

    #[test_log::test]
    #[should_panic]
    fn add_package_without_name() {
        let (_tempdir, index) = create_test_index();

        let mut package_test = PackageTest::default();
        package_test.name = "".to_string();

        index.add_package(&package_test).unwrap();
    }

    #[test_log::test]
    #[should_panic]
    fn add_package_without_version() {
        let (_tempdir, index) = create_test_index();

        let mut package_test = PackageTest::default();
        package_test.version = "".to_string();

        index.add_package(&package_test).unwrap();
    }

    #[test_log::test]
    #[should_panic]
    fn add_package_without_repository_name() {
        let (_tempdir, index) = create_test_index();

        let mut package_test = PackageTest::default();
        package_test.repository_name = "".to_string();

        index.add_package(&package_test).unwrap();
    }

    #[test_log::test]
    #[should_panic]
    fn add_package_without_hashsum() {
        let (_tempdir, index) = create_test_index();

        let mut package_test = PackageTest::default();
        package_test.hashsum = "".to_string();

        index.add_package(&package_test).unwrap();
    }

    #[test_log::test]
    fn delete_package() {
        let (_tempdir, index) = create_test_index();

        let package_test = PackageTest::default();

        index.add_package(&package_test).unwrap();
        index.delete_package(&package_test.name).unwrap();
    }

    #[test_log::test]
    #[should_panic]
    fn delete_no_existent_package() {
        let (_tempdir, index) = create_test_index();

        index.delete_package("not package").unwrap();
    }
}
