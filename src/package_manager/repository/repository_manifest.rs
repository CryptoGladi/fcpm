pub mod package_manifest;

use crate::{package::Package, package_manager::repository::RepositoryError};
use package_manifest::PackageManifest;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{fmt::Debug, fs::OpenOptions, io::Write, path::Path};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryManifest<P>
where
    P: Package,
{
    pub name: String,
    pub url: String,
    pub packages: Vec<PackageManifest<P>>,
}

impl<P> RepositoryManifest<P>
where
    P: Package,
{
    pub fn add_package(&mut self, package: P) {
        #[cfg(feature = "logging")]
        log::debug!("Add package: `{}`", package.name());

        // TODO Check same name and version

        let package_manifest = PackageManifest::from(package);
        self.packages.push(package_manifest);
    }

    #[must_use]
    pub fn get_package(&self, package_name: &str) -> Option<&PackageManifest<P>> {
        #[cfg(feature = "logging")]
        log::debug!("Get package from name: {package_name}");

        self.packages
            .iter()
            .find(|package| package.name == package_name)
    }
}

impl<P> RepositoryManifest<P>
where
    P: Package + Serialize,
{
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
}

impl<P> RepositoryManifest<P>
where
    P: Package + DeserializeOwned,
{
    pub fn parse(manifest: &str) -> Result<Self, RepositoryError> {
        #[cfg(feature = "logging")]
        log::debug!("Parse from string repository manifest");

        Ok(serde_json::from_str(manifest)?)
    }
}

impl<P> std::str::FromStr for RepositoryManifest<P>
where
    P: Package + DeserializeOwned,
{
    type Err = RepositoryError;

    fn from_str(manifest: &str) -> Result<Self, Self::Err> {
        Self::parse(manifest)
    }
}

impl<P> std::fmt::Display for RepositoryManifest<P>
where
    P: Package + Serialize,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(&self).expect("Serialize to json error")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::example::package::PackageExample;
    use std::str::FromStr;
    use tempfile::tempdir;

    pub(crate) type RepositoryManifestTest = RepositoryManifest<PackageExample>;

    #[test_log::test]
    fn add_get_package() {
        let mut repository_manifest = RepositoryManifestTest::default();
        let package = PackageExample::default();

        repository_manifest.add_package(package.clone());

        assert_eq!(
            repository_manifest.get_package(&package.name),
            Some(&PackageManifest::from(package))
        );
    }

    #[test_log::test]
    fn save_to_file() {
        let mut repository_manifest = RepositoryManifestTest::default();
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
        let mut repository_manifest = RepositoryManifestTest::default();
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
