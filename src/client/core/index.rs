use crate::package::Package;
use rusqlite::Connection;
use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};
use thiserror::Error;

/// A `SQLite`-based index for managing packages.
///
/// The `Index` struct provides an interface to store, retrieve, and manage packages
/// in a `SQLite` database. It supports operations like adding, deleting, and querying packages.
///
/// # Type Parameters
///
/// * `P` - The type of package, must implement [`Package`].
///
/// # Examples
///
/// ```no_run
/// use fcpm::Index;
/// use rusqlite::OpenFlags;
/// # use fcpm::example::package::PackageExample as Package;
///
/// let mut index: Index<Package> = Index::open("index.sqlite", OpenFlags::default())?;
///
/// let some_package = Package::default();
/// index.add_package(&some_package)?;
/// # Ok::<(), fcpm::client::core::index::IndexError>(())
/// ```
#[derive(Debug)]
pub struct Index<P>
where
    P: Package,
{
    /// The path to the `SQLite` database file.
    path: PathBuf,

    /// The `SQLite` database connection.
    db: rusqlite::Connection,

    phantom: PhantomData<P>,
}

impl<P> PartialEq for Index<P>
where
    P: Package,
{
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl<P> Eq for Index<P> where P: Package {}

/// Errors that can occur when interacting with the index.
#[derive(Debug, Error)]
pub enum IndexError {
    /// A `SQLite` database error occurred.
    #[error("SQLite error: `{0}`")]
    SQLite(#[from] rusqlite::Error),

    /// A JSON serialization/deserialization error occurred.
    #[error("Json error: `{0}`")]
    Json(#[from] serde_json::Error),

    /// The specified package was not found.
    #[error("Package `{0}` not found")]
    PackageNotFound(String),
}

impl<P> Index<P>
where
    P: Package,
{
    /// Opens an index at the specified path with the given `SQLite` open flags.
    /// This method initializes the `SQLite` database and creates the necessary tables if they don't exist.
    ///
    /// # Parameters
    ///
    /// * `path` - The path to the `SQLite` database file.
    /// * `open_flags` - The flags to use when opening the database.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the opened `Index` or an [`IndexError`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fcpm::Index;
    /// use rusqlite::OpenFlags;
    /// # use fcpm::example::package::PackageExample as Package;
    ///
    /// let index: Index<Package> = Index::open("index.sqlite", OpenFlags::default())?;
    /// # Ok::<(), fcpm::client::core::index::IndexError>(())
    /// ```
    pub fn open(
        path: impl AsRef<Path>,
        open_flags: rusqlite::OpenFlags,
    ) -> Result<Self, IndexError> {
        let path_buf = path.as_ref().to_path_buf();

        #[cfg(feature = "logging")]
        log::debug!("Open index to path: {}", path_buf.display());

        let db = Connection::open_with_flags(path, open_flags)?;
        let mut index = Self {
            path: path_buf,
            db,
            phantom: PhantomData,
        };

        index.init()?;

        Ok(index)
    }

    fn init(&mut self) -> Result<(), IndexError> {
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

    /// Adds a package to the index.
    ///
    /// # Parameters
    ///
    /// * `package` - The package to add, must implement [`Package`].
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or an [`IndexError`] on failure.
    ///
    /// # Errors
    ///
    /// Returns [`IndexError::SQLite`] if a database error occurs.
    /// Returns [`IndexError::Json`] if metadata serialization fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fcpm::Index;
    /// use rusqlite::OpenFlags;
    /// # use fcpm::example::package::PackageExample as Package;
    ///
    /// let mut index: Index<Package> = Index::open("index.sqlite", OpenFlags::default())?;
    /// let package = Package::default();
    ///
    /// index.add_package(&package)?;
    /// # Ok::<(), fcpm::client::core::index::IndexError>(())
    /// ```
    pub fn add_package(&mut self, package: &impl Package) -> Result<(), IndexError> {
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

    /// Deletes a package from the index by name.
    ///
    /// # Parameters
    ///
    /// * `package_name` - The name of the package to delete.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or an [`IndexError`] on failure.
    ///
    /// # Errors
    ///
    /// Returns [`IndexError::PackageNotFound`] if the package does not exist.
    /// Returns [`IndexError::SQLite`] if a database error occurs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fcpm::Index;
    /// # use fcpm::example::package::PackageExample as Package;
    /// # use rusqlite::OpenFlags;
    /// #
    /// let mut index: Index<Package> = Index::open("index.sqlite", OpenFlags::default())?;
    ///
    /// index.delete_package("my-package")?;
    /// #
    /// # Ok::<(), fcpm::client::core::index::IndexError>(())
    /// ```
    pub fn delete_package(&mut self, package_name: &str) -> Result<(), IndexError> {
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

    /// Retrieves a package from the index by name.
    ///
    /// # Parameters
    ///
    /// * `package_name` - The name of the package to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the package or an [`IndexError`].
    ///
    /// # Errors
    ///
    /// Returns [`IndexError::SQLite`] if a database error occurs or the package is not found.
    /// Returns [`IndexError::Json`] if metadata deserialization fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fcpm::Index;
    /// use rusqlite::OpenFlags;
    /// use fcpm::Package;
    /// # use fcpm::example::package::PackageExample;
    ///
    /// let index: Index<PackageExample> = Index::open("index.sqlite", OpenFlags::default())?;
    /// let package = index.get_package("my-package")?;
    ///
    /// assert_eq!(package.name(), "package-name");
    /// # Ok::<(), fcpm::client::core::index::IndexError>(())
    /// ```
    pub fn get_package(&self, package_name: &str) -> Result<P, IndexError> {
        #[cfg(feature = "logging")]
        log::debug!("Get package by name: {package_name}");

        let (version, repository_name, hashsum, metadata_str) = self.db.query_one(
            "select `version`, `repository_name`, `hashsum`, `metadata` from `packages` where name = ?1",
            ((package_name),),
            |row| {
                let version = row.get(0)?;
                let repository_name = row.get(1)?;
                let hashsum = row.get(2)?;
                let metadata: String = row.get(3)?;

                Ok((version, repository_name, hashsum, metadata))
            },
        )?;

        let metadata = serde_json::from_str(&metadata_str)?;
        let package = P::new(
            package_name.to_string(),
            version,
            repository_name,
            hashsum,
            metadata,
        );

        Ok(package)
    }

    /// Retrieves all packages from the index.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of packages or an [`IndexError`].
    ///
    /// # Errors
    ///
    /// Returns [`IndexError::SQLite`] if a database error occurs.
    /// Returns [`IndexError::Json`] if metadata deserialization fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fcpm::Index;
    /// use rusqlite::OpenFlags;
    /// # use fcpm::example::package::PackageExample as Package;
    ///
    /// let index: Index<Package> = Index::open("index.sqlite", OpenFlags::default())?;
    ///
    /// let packages: Vec<Package> = index.get_packages()?;
    /// # Ok::<(), fcpm::client::core::index::IndexError>(())
    /// ```
    pub fn get_packages(&self) -> Result<Vec<P>, IndexError> {
        #[cfg(feature = "logging")]
        log::debug!("Get all packages");

        let mut stmt = self.db.prepare(
            "select `name`, `version`, `repository_name`, `hashsum`, `metadata` from `packages`",
        )?;

        let rows = stmt.query_map((), |row| {
            let name: String = row.get(0)?;
            let version: String = row.get(1)?;
            let repository_name: String = row.get(2)?;
            let hashsum: String = row.get(3)?;
            let metadata: String = row.get(4)?;

            Ok((name, version, repository_name, hashsum, metadata))
        })?;

        let mut packages = Vec::new();
        for row in rows {
            let (name, version, repository_name, hashsum, metadata) = row?;
            let metadata = serde_json::from_str(&metadata)?;

            let package = P::new(name, version, repository_name, hashsum, metadata);
            packages.push(package);
        }

        Ok(packages)
    }

    /// Checks if a package exists in the index.
    ///
    /// # Parameters
    ///
    /// * `package_name` - The name of the package to check.
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if the package exists, `Ok(false)` if it does not, or an [`IndexError`] on failure.
    ///
    /// # Errors
    ///
    /// Returns [`IndexError::SQLite`] if a database error occurs (other than not found).
    /// Returns [`IndexError::Json`] if metadata deserialization fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fcpm::Index;
    /// # use fcpm::example::package::PackageExample as Package;
    /// # use rusqlite::OpenFlags;
    /// #
    /// let index: Index<Package> = Index::open("index.sqlite", OpenFlags::default())?;
    /// let exists = index.have_package("my-package")?;
    ///
    /// assert!(exists);
    /// # Ok::<(), fcpm::client::core::index::IndexError>(())
    /// ```
    pub fn have_package(&self, package_name: &str) -> Result<bool, IndexError> {
        #[cfg(feature = "logging")]
        log::debug!("Have package: {package_name}?");

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
    use crate::example::package::{PackageExample, PackageExampleWithMetadata};
    use rusqlite::OpenFlags;
    use tempfile::{TempDir, tempdir};

    pub(crate) type IndexTest = Index<PackageExample>;
    pub(crate) type IndexTestWithMetadata = Index<PackageExampleWithMetadata>;

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
        let (_tempdir, mut index) = create_test_index();

        index.init().unwrap();
    }

    #[test_log::test]
    fn add_package() {
        let (_tempdir, mut index) = create_test_index();
        let package_test = PackageExample::default();

        index.add_package(&package_test).unwrap();
    }

    #[test_log::test]
    fn add_package_with_metadata() {
        let (_tempdir, mut index) = create_test_index_with_metadata();
        let package_test = PackageExampleWithMetadata::default();

        index.add_package(&package_test).unwrap();
    }

    #[test_log::test]
    #[should_panic]
    fn add_package_with_same_name() {
        let (_tempdir, mut index) = create_test_index();

        let package_test1 = PackageExample {
            name: "supername".to_string(),
            ..Default::default()
        };
        let package_test2 = PackageExample {
            name: "supername".to_string(),
            ..Default::default()
        };

        index.add_package(&package_test1).unwrap();
        index.add_package(&package_test2).unwrap(); // Panic
    }

    #[test_log::test]
    #[should_panic]
    fn add_package_without_name() {
        let (_tempdir, mut index) = create_test_index();

        let mut package_test = PackageExample::default();
        package_test.name = "".to_string();

        index.add_package(&package_test).unwrap();
    }

    #[test_log::test]
    fn add_package_name_with_unicode() {
        let (_tempdir, mut index) = create_test_index();

        let mut package_test = PackageExample::default();
        package_test.name = "package-name-with-unicode-😅😅😅".to_string();
        index.add_package(&package_test).unwrap();

        assert_eq!(index.get_packages().unwrap(), [package_test]);
    }

    #[test_log::test]
    #[should_panic]
    fn add_package_without_version() {
        let (_tempdir, mut index) = create_test_index();

        let mut package_test = PackageExample::default();
        package_test.version = "".to_string();

        index.add_package(&package_test).unwrap();
    }

    #[test_log::test]
    #[should_panic]
    fn add_package_without_repository_name() {
        let (_tempdir, mut index) = create_test_index();

        let mut package_test = PackageExample::default();
        package_test.repository_name = "".to_string();

        index.add_package(&package_test).unwrap();
    }

    #[test_log::test]
    #[should_panic]
    fn add_package_without_hashsum() {
        let (_tempdir, mut index) = create_test_index();

        let mut package_test = PackageExample::default();
        package_test.hashsum = "".to_string();

        index.add_package(&package_test).unwrap();
    }

    #[test_log::test]
    fn delete_package() {
        let (_tempdir, mut index) = create_test_index();

        let package_test = PackageExample::default();

        index.add_package(&package_test).unwrap();
        index.delete_package(&package_test.name).unwrap();
    }

    #[test_log::test]
    #[should_panic]
    fn delete_no_existent_package() {
        let (_tempdir, mut index) = create_test_index();

        index.delete_package("not package").unwrap(); // PANIC
    }

    #[test_log::test]
    fn get_package() {
        let (_tempdir, mut index) = create_test_index();

        let package = PackageExample::default();
        index.add_package(&package).unwrap();

        let got_package = index.get_package(&package.name).unwrap();
        assert_eq!(package, got_package);
    }

    #[test_log::test]
    fn get_package_with_metadata() {
        let (_tempdir, mut index) = create_test_index_with_metadata();

        let package = PackageExampleWithMetadata::default();
        index.add_package(&package).unwrap();

        let got_package = index.get_package(&package.name).unwrap();
        assert_eq!(package, got_package);
        assert_eq!(package.metadata, got_package.metadata);
    }

    #[test_log::test]
    #[should_panic]
    fn get_package_not_found() {
        let (_tempdir, mut index) = create_test_index();

        let package = PackageExample::default();
        index.add_package(&package).unwrap();

        assert_ne!(package.name, "is_not_package");
        index.get_package("is_not_package").unwrap(); // PANIC
    }

    #[test_log::test]
    fn add_get_delete_package() {
        let (_tempdir, mut index) = create_test_index();
        let package = PackageExample::default();

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
        let (_tempdir, mut index) = create_test_index();
        let package = PackageExample::default();

        index.add_package(&package).unwrap();
        assert!(index.have_package(&package.name).unwrap());
    }

    #[test_log::test]
    fn get_packages() {
        let (_tempdir, mut index) = create_test_index();

        assert_eq!(index.get_packages().unwrap(), []);

        let package_test = PackageExample::default();
        index.add_package(&package_test).unwrap();

        assert_eq!(index.get_packages().unwrap(), [package_test]);
    }

    #[test_log::test]
    fn get_packages_with_metadata() {
        let (_tempdir, mut index) = create_test_index_with_metadata();

        assert_eq!(index.get_packages().unwrap(), []);

        let package = PackageExampleWithMetadata::default();
        index.add_package(&package).unwrap();

        assert_eq!(index.get_packages().unwrap(), [package]);
    }

    #[test_log::test]
    fn partial_eq() {
        let (_tempdir1, index1) = create_test_index();
        let (_tempdir2, index2) = create_test_index();

        assert_eq!(index1, index1);
        assert_eq!(index2, index2);
        assert_ne!(index1, index2);
    }
}
