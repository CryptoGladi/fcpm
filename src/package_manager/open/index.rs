use rusqlite::Connection;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct Index {
    path: PathBuf,
    db: rusqlite::Connection,
}

impl Index {
    pub(crate) fn open(
        path: impl AsRef<Path>,
        open_flags: rusqlite::OpenFlags,
    ) -> Result<Self, rusqlite::Error> {
        let path_buf = path.as_ref().to_path_buf();

        #[cfg(feature = "logging")]
        log::debug!("Open index to path: {}", path_buf.display());

        let db = Connection::open_with_flags(path, open_flags)?;
        let index = Self { path: path_buf, db };

        index.init()?;

        Ok(index)
    }

    fn init(&self) -> Result<(), rusqlite::Error> {
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
                    `hashsum` varchar(256) null
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test]
    fn t() {}
}
