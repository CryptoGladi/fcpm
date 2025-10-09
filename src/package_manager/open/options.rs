#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub index_name: String,
    pub lockfile_name: String,
    pub repository_name: String,
    pub create_if_not_exists: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            index_name: "index.sqlite".to_string(),
            lockfile_name: "fcpm.lock".to_string(),
            repository_name: "repo".to_string(),
            create_if_not_exists: true,
        }
    }
}

impl From<Options> for rusqlite::OpenFlags {
    fn from(value: Options) -> Self {
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

macro_rules! impl_get_fn {
    ($fn_name:ident, $value:ident) => {
        pub fn $fn_name(&self, path: impl AsRef<std::path::Path>) -> std::path::PathBuf {
            path.as_ref().join(&self.$value)
        }
    };
}

impl Options {
    impl_get_fn!(path_index, index_name);
    impl_get_fn!(path_lockfile, lockfile_name);
    impl_get_fn!(path_repository, repository_name);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test_log::test]
    fn from_rusqlite() {
        use rusqlite::OpenFlags;

        let mut options = Options::default();
        options.create_if_not_exists = false;
        let sqlite_options: OpenFlags = options.into();

        assert_eq!(
            sqlite_options,
            OpenFlags::SQLITE_OPEN_NO_MUTEX
                | OpenFlags::SQLITE_OPEN_URI
                | OpenFlags::SQLITE_OPEN_READ_WRITE
        );
    }

    #[test_log::test]
    fn from_rusqlite_with_create_in_open() {
        use rusqlite::OpenFlags;

        let mut options = Options::default();
        options.create_if_not_exists = true;

        let sqlite_options: OpenFlags = options.into();

        assert_eq!(
            sqlite_options,
            OpenFlags::SQLITE_OPEN_NO_MUTEX
                | OpenFlags::SQLITE_OPEN_URI
                | OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
        );
    }

    #[test_log::test]
    fn path_to_index() {
        let options = Options::default();

        assert_eq!(
            options.path_index("folder"),
            PathBuf::from("folder").join(options.index_name)
        );
    }

    #[test_log::test]
    fn path_to_lockfile() {
        let options = Options::default();

        assert_eq!(
            options.path_lockfile("folder"),
            PathBuf::from("folder").join(options.lockfile_name)
        );
    }

    #[test_log::test]
    fn path_to_repository_name() {
        let options = Options::default();

        assert_eq!(
            options.path_repository("folder"),
            PathBuf::from("folder").join(options.repository_name)
        );
    }
}
