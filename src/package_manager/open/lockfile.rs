use std::fs;
use std::path::PathBuf;
use std::{fs::File, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LockFileError {
    #[error("IO error: `{0}`")]
    IO(#[from] std::io::Error),
}

#[derive(Debug)]
pub(crate) struct LockFile {
    file: File,
    path: PathBuf,
}

impl LockFile {
    pub(crate) fn new(path: impl AsRef<Path>) -> Result<Self, LockFileError> {
        let path_buf = path.as_ref().to_path_buf();

        #[cfg(feature = "logging")]
        log::debug!("Create lock file: `{}`", path_buf.display());

        let file = fs::OpenOptions::new().append(true).open(&path_buf)?;
        file.lock()?;

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

    #[test_log::test]
    fn lock() {
        let lock = LockFile::new("ld").unwrap();
        drop(lock);
    }

    #[test_log::test]
    fn unlock() {
        let path = "ds";

        let lock1 = LockFile::new(path).unwrap();
        lock1.unlock().unwrap();

        let lock2 = LockFile::new(path).unwrap();
        drop(lock2);
    }

    #[test_log::test]
    #[should_panic]
    fn conflict_lock() {
        let path = "d";

        let _lock1 = LockFile::new(path).unwrap();
        let _lock2 = LockFile::new(path).unwrap(); // Panic
    }
}
