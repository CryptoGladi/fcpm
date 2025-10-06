use crate::package::Package;
use log::Metadata;
use rusqlite::Connection;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug)]
pub struct Index<Metadata, P>
where
    Metadata: Serialize + DeserializeOwned + Clone,
    P: Package<Metadata>,
{
    path: PathBuf,
    db: rusqlite::Connection,
    phantom: (PhantomData<Metadata>, PhantomData<P>),
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

impl<Metadata, P> Index<Metadata, P>
where
    Metadata: Serialize + DeserializeOwned + Clone,
    P: Package<Metadata>,
{
    pub fn open(
        path: impl AsRef<Path>,
        open_flags: rusqlite::OpenFlags,
    ) -> Result<Self, IndexError> {
        let path_buf = path.as_ref().to_path_buf();

        #[cfg(feature = "logging")]
        log::debug!("Open index to path: {}", path_buf.display());

        let db = Connection::open_with_flags(path, open_flags)?;
        let index = Self {
            path: path_buf,
            db,
            phantom: (PhantomData, PhantomData),
        };

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

    pub fn add_package(&self, package: &impl Package<Metadata>) -> Result<(), IndexError> {
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

    pub fn get_package(&self, package_name: &str) -> Result<P, IndexError> {
        let package = self.db.query_one(
            "select `version`, `repository_name`, `hashsum`, `metadata` from `packages` where name = ?1",
            ((package_name),),
            |row| {
                let version = row.get(0)?;
                let repository_name = row.get(1)?;
                let hashsum = row.get(2)?;

                // TODO delete unwrap
                let metadata: String = row.get(3)?;
                let metadata = serde_json::from_str(&metadata).unwrap();

                Ok(P::new(
                    package_name.to_string(),
                    version,
                    repository_name,
                    hashsum,
                    metadata,
                ))
            },
        )?;

        Ok(package)
    }

    pub fn get_packages(&self) -> Result<(), IndexError> {
        // TODO ????
        let packages = self.db.query_row(
            "select `name`, `version`, `repository_name`, `hashsum`, `metadata` from `packages`",
            (),
            |row| {
                let name = row.get(0)?;
                let version = row.get(1)?;
                let repository_name = row.get(2)?;
                let hashsum = row.get(3)?;

                // TODO delete unwrap
                let metadata: String = row.get(4)?;
                let metadata = serde_json::from_str(&metadata).unwrap();

                Ok(P::new(name, version, repository_name, hashsum, metadata))
            },
        )?;
        Ok(())
    }

    pub fn have_package(&self, package_name: &str) -> Result<bool, IndexError> {
        match self.get_package(package_name) {
            Err(IndexError::SQLite(rusqlite::Error::QueryReturnedNoRows)) => Ok(false),
            Err(error) => Err(error),
            Ok(_) => Ok(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::tests::{MetadataTest, PackageTest, PackageTestWithMetadata};
    use rusqlite::OpenFlags;
    use tempfile::{TempDir, tempdir};

    pub(crate) type IndexTest = Index<(), PackageTest>;
    pub(crate) type IndexTestWithMetadata = Index<MetadataTest, PackageTestWithMetadata>;

    pub(crate) fn create_test_index() -> (TempDir, IndexTest) {
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("index.sqlite");

        let index = Index::open(&path, OpenFlags::default()).unwrap();
        (tempdir, index)
    }

    pub(crate) fn create_test_index_with_metadata() -> (TempDir, IndexTestWithMetadata) {
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("index.sqlite");

        let index = Index::open(&path, OpenFlags::default()).unwrap();
        (tempdir, index)
    }

    #[test_log::test]
    fn open() {
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("index.sqlite");

        let index: IndexTest = Index::open(&path, OpenFlags::default()).unwrap();

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
    }

    #[test_log::test]
    fn add_package_with_metadata() {
        let (_tempdir, index) = create_test_index_with_metadata();
        let package_test = PackageTestWithMetadata::default();

        index.add_package(&package_test).unwrap();
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

        index.delete_package("not package").unwrap(); // PANIC
    }

    #[test_log::test]
    fn get_package() {
        let (_tempdir, index) = create_test_index();

        let package = PackageTest::default();
        index.add_package(&package).unwrap();

        let got_package = index.get_package(&package.name).unwrap();
        assert_eq!(package, got_package);
    }

    #[test_log::test]
    fn get_package_with_metadata() {
        let (_tempdir, index) = create_test_index_with_metadata();

        let package = PackageTestWithMetadata::default();
        index.add_package(&package).unwrap();

        let got_package = index.get_package(&package.name).unwrap();
        assert_eq!(package, got_package);
        assert_eq!(package.metadata, got_package.metadata);
    }

    #[test_log::test]
    #[should_panic]
    fn get_package_not_found() {
        let (_tempdir, index) = create_test_index();

        let package = PackageTest::default();
        index.add_package(&package).unwrap();

        assert_ne!(package.name, "is_not_package");
        index.get_package("is_not_package").unwrap(); // PANIC
    }

    #[test_log::test]
    fn add_get_delete_package() {
        let (_tempdir, index) = create_test_index();
        let package = PackageTest::default();

        index.add_package(&package).unwrap();
        let got_package = index.get_package(&package.name).unwrap();

        assert_eq!(package, got_package);
        index.delete_package(&package.name).unwrap();

        let result = index.get_package(&package.name);
        assert!(matches!(
            result,
            Err(IndexError::SQLite(rusqlite::Error::QueryReturnedNoRows))
        ));
    }

    #[test_log::test]
    fn have_package() {
        let (_tempdir, index) = create_test_index();
        let package = PackageTest::default();

        index.add_package(&package).unwrap();
        assert!(index.have_package(&package.name).unwrap());
    }
}
