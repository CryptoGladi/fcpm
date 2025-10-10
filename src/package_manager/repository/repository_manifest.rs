use crate::{package::Package, package_manager::repository::RepositoryError};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{fs::OpenOptions, io::Write, marker::PhantomData, path::Path};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryManifest<P>
where
    P: Package,
{
    pub name: String,
    pub url: String,
    pub packages: Vec<P>,
}

/*
impl RepositoryManifest {
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), RepositoryError> {
        #[cfg(feature = "logging")]
        log::debug!(
            "Save to file repository manifest to `{}` with {} packages",
            path.as_ref().display(),
            self.packages.len()
        );

        let json = serde_json::to_string(&self)?;
        let mut file = OpenOptions::new().append(true).create(true).open(path)?;
        file.write_all(json.as_bytes())?;
        drop(file); // Close file

        Ok(())
    }

    pub fn parse(manifest: &str) -> Result<Self, RepositoryError> {
        #[cfg(feature = "logging")]
        log::debug!("Parse from string repository manifest");

        Ok(serde_json::from_str(manifest)?)
    }
}

impl std::str::FromStr for RepositoryManifest {
    type Err = RepositoryError;

    fn from_str(manifest: &str) -> Result<Self, Self::Err> {
        Self::parse(manifest)
    }
}

impl std::string::ToString for RepositoryManifest {
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("Serde error")
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use tempfile::tempdir;

    #[test_log::test]
    fn save_to_file() {
        let mut repository_manifest = RepositoryManifest::default();
        repository_manifest.name = "test-repo".to_string();

        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("manifest.json");
        repository_manifest.save_to_file(&path).unwrap();

        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            r#"{"name":"test-repo","url":"","packages":[]}"#
        );
    }

    #[test_log::test]
    fn parse() {
        let mut repository_manifest = RepositoryManifest::default();
        repository_manifest.name = "test-repo".to_string();

        let json = repository_manifest.to_string();

        assert_eq!(
            RepositoryManifest::parse(&json).unwrap(),
            repository_manifest
        );

        assert_eq!(
            RepositoryManifest::from_str(&json).unwrap(),
            repository_manifest
        );
    }
}
*/
