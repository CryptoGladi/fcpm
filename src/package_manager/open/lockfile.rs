use std::fs;
use std::path::PathBuf;
use std::{fs::File, path::Path};
use thiserror::Error;

/// Errors that can occur when working with the lockfile.
#[derive(Debug, Error)]
pub enum LockFileError {
    /// An I/O error occurred.
    #[error("IO error: `{0}`")]
    IO(#[from] std::io::Error),

    /// A file locking error occurred.
    #[error("Lock error: `{0}`")]
    Lock(#[from] std::fs::TryLockError),
}

/// A file-based lock to prevent concurrent access to a resource.
///
/// The `LockFile` creates a lock file and uses file locking to ensure exclusive access.
/// When the `LockFile` is dropped, the lock is released and the file is removed.
///
/// # Examples
///
/// ```no_run
/// use fcpm::LockFile;
///
/// let lock = LockFile::new("my.lock")?;
/// # Ok::<(), fcpm::package_manager::open::lockfile::LockFileError>(())
/// ```
#[derive(Debug)]
pub struct LockFile {
    /// The locked file handle.
    file: File,

    /// The path to the lock file.
    path: PathBuf,
}

impl PartialEq for LockFile {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for LockFile {}

impl LockFile {
    /// Creates a new lockfile at the specified path.
    ///
    /// This method creates the lock file if it doesn't exist, opens it for appending,
    /// and acquires an exclusive lock on it.
    ///
    /// # Parameters
    ///
    /// * `path` - The path where the lock file should be created.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `LockFile` or a [`LockFileError`].
    ///
    /// # Errors
    ///
    /// Returns [`LockFileError::IO`] if the file cannot be created or opened.
    /// Returns [`LockFileError::Lock`] if the lock cannot be acquired.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fcpm::LockFile;
    ///
    /// let lock = LockFile::new("my.lock")?;
    /// # Ok::<(), fcpm::package_manager::open::lockfile::LockFileError>(())
    /// ```
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
            path: path_buf.clone(),
        })
    }

    /// Releases the lock and removes the lock file.
    ///
    /// This method unlocks the file and deletes it from the filesystem.
    /// It is automatically called when the `LockFile` is dropped.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or a [`LockFileError`] on failure.
    ///
    /// # Errors
    ///
    /// Returns [`LockFileError::Lock`] if unlocking fails.
    /// Returns [`LockFileError::IO`] if removing the file fails.
    fn unlock(&self) -> Result<(), LockFileError> {
        #[cfg(feature = "logging")]
        log::debug!("Unlocking log file: `{}`", self.path.display());

        self.file.unlock()?;
        fs::remove_file(&self.path)?;

        Ok(())
    }
}

impl Drop for LockFile {
    #[cfg_attr(not(feature = "logging"), allow(unused_variables))]
    fn drop(&mut self) {
        if let Err(unlock_error) = self.unlock() {
            #[cfg(feature = "logging")]
            log::debug!("Error unlock lockfile in drop: `{unlock_error}`");
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
