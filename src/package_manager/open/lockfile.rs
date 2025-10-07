use std::fs;
use std::path::PathBuf;
use std::{fs::File, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LockFileError {
    #[error("IO error: `{0}`")]
    IO(#[from] std::io::Error),

    #[error("Lock error: `{0}`")]
    Lock(#[from] std::fs::TryLockError),
}

#[derive(Debug)]
pub struct LockFile {
    file: File,
    path: PathBuf,
}

impl PartialEq for LockFile {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for LockFile {}

impl LockFile {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, LockFileError> {
        let path_buf = path.as_ref().to_path_buf();

        #[cfg(feature = "logging")]
        log::debug!("Create lock file: `{}`", path_buf.display());

        let file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&path_buf)?;

        file.try_lock()?;

        Ok(Self {
            file,
            path: path_buf.to_path_buf(),
        })
    }

    fn unlock(&self) -> Result<(), LockFileError> {
        #[cfg(feature = "logging")]
        log::debug!("Unlocking log file: `{}`", self.path.display());

        self.file.unlock()?;
        fs::remove_file(&self.path)?;

        Ok(())
    }
}

impl Drop for LockFile {
    fn drop(&mut self) {
        if let Err(unlock_error) = self.unlock() {
            #[cfg(feature = "logging")]
            log::debug!("Error unlock lockfile in drop: `{}`", unlock_error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test_log::test]
    fn lock() {
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("lock");

        let lock = LockFile::new(path).unwrap();
        drop(lock);
    }

    #[test_log::test]
    fn unlock() {
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("lock");

        let lock1 = LockFile::new(&path).unwrap();
        lock1.unlock().unwrap();

        let lock2 = LockFile::new(&path).unwrap();
        drop(lock2);
    }

    #[test_log::test]
    #[should_panic]
    fn conflict_lock() {
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("lock");

        let _lock1 = LockFile::new(&path).unwrap();
        let _lock2 = LockFile::new(&path).unwrap(); // Panic
    }

    #[test_log::test]
    fn partial_eq() {
        let tempdir = tempdir().unwrap();

        let path1 = tempdir.path().join("lock1");
        let lock1 = LockFile::new(&path1).unwrap();

        let path2 = tempdir.path().join("lock2");
        let lock2 = LockFile::new(&path2).unwrap();

        assert_eq!(lock1, lock1);
        assert_eq!(lock2, lock2);
        assert_ne!(lock1, lock2);
    }
}
