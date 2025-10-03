use crate::package::Package;
use rusqlite::Connection;
use serde::{Serialize, de::DeserializeOwned};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug)]
pub(crate) struct Index {
    path: PathBuf,
    db: rusqlite::Connection,
}

#[derive(Debug, Error)]
pub enum IndexError {
    #[error("SQLite error: `{0}`")]
    SQLite(#[from] rusqlite::Error),
}

impl Index {
    pub(crate) fn open(
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
        log::trace!("Create table `packages`");
        self.db.execute(
            "create table `packages` if NOT exists (
                    `id` integer not null primary key autoincrement,
                    `package-name` varchar(255) not null unique,
                    `package-path` varchar(255) not null unique,
                    `version` varchar(255) not null,
                    `repository-name` varchar(255) not null,
                    `hashsum` varchar(256) not null
                    `metadata` text null
                )",
            (),
        )?;

        #[cfg(feature = "logging")]
        log::trace!("Create index for table `packages`");
        self.db.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS package-name-idx ON packages(package-name)",
            (),
        )?;

        Ok(())
    }

    pub(crate) fn add_package<'a, M, P>(&self, package: P) -> Result<(), IndexError>
    where
        M: Serialize + DeserializeOwned,
        P: Package<'a, M>,
    {
        // TODO
        //self.db.execute("insert into table ", params)
        Ok(())
    }

    pub(crate) fn delete_package_by_name(&self, package_name: &str) -> Result<(), IndexError> {
        Ok(())
    }

    pub(crate) fn delete_package_by_path(
        &self,
        package_path: impl AsRef<Path>,
    ) -> Result<(), IndexError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::OpenFlags;

    #[test_log::test]
    fn open() {
        let path = PathBuf::from("");
        let index = Index::open(&path, OpenFlags::SQLITE_OPEN_CREATE).unwrap();

        assert_eq!(index.path, path);
    }

    #[test_log::test]
    fn init() {
        let path = "";
        let index = Index::open(path, OpenFlags::SQLITE_OPEN_CREATE).unwrap();

        index.init().unwrap();
    }
}
