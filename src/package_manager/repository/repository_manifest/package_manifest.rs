use crate::package::Package;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PackageManifest<P>
where
    P: Package,
{
    pub name: String,
    pub version: String,
    pub hashsum: String,
    pub metadata: P::Metadata,
}

impl<P> From<P> for PackageManifest<P>
where
    P: Package,
{
    fn from(value: P) -> Self {
        Self {
            name: value.name().into_owned(),
            version: value.version().into_owned(),
            hashsum: value.hashsum().into_owned(),
            metadata: value.metadata().into_owned(),
        }
    }
}

impl<P> std::fmt::Debug for PackageManifest<P>
where
    P: Package,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PackageManifest")
            .field(&self.name)
            .field(&self.version)
            .field(&self.hashsum)
            .finish()
    }
}

/// # Warning
/// Dont check metadata
impl<P> PartialEq for PackageManifest<P>
where
    P: Package,
{
    fn eq(&self, other: &Self) -> bool {
        (&self.name, &self.version, &self.hashsum) == (&other.name, &other.version, &other.hashsum)
    }
}

impl<P> Eq for PackageManifest<P> where P: Package {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::tests::{PackageTest, PackageTestWithMetadata};

    pub(crate) type PackageManifestTest = PackageManifest<PackageTest>;
    pub(crate) type PackageManifestTestWithMetadata = PackageManifest<PackageTestWithMetadata>;

    #[test_log::test]
    fn from_package() {
        let package_test = PackageTest::default();
        let package_test_manifest = PackageManifest::from(package_test.clone());

        assert_eq!(
            (
                package_test.name().as_ref(),
                package_test.version().as_ref(),
                package_test.hashsum().as_ref(),
                package_test.metadata().as_ref()
            ),
            (
                package_test_manifest.name.as_ref(),
                package_test_manifest.version.as_ref(),
                package_test_manifest.hashsum.as_ref(),
                &package_test_manifest.metadata
            )
        );
    }

    #[test_log::test]
    fn debug() {
        let p1 = PackageManifest::from(PackageTest::default());
        let p2 = PackageManifest::from(PackageTestWithMetadata::default());

        assert_eq!(
            format!("{p1:?}"),
            r#"PackageManifest("test", "0.1.0", "test-sha256")"#
        );

        assert_eq!(
            format!("{p2:?}"),
            r#"PackageManifest("test-with-metadata", "0.1.0", "test-sha256")"#
        );
    }

    #[test_log::test]
    fn partical_eq() {
        let p1 = PackageManifest::from(PackageTest::default());
        let mut p2 = PackageManifest::from(PackageTest::default());

        p2.name = "123".to_string();

        assert_eq!(p1, p1);
        assert_eq!(p2, p2);

        assert_ne!(p1, p2);
        assert_ne!(p2, p1);
    }

    #[test_log::test]
    fn partical_eq_with_metadata() {
        let p1 = PackageManifest::from(PackageTestWithMetadata::default());
        let mut p2 = PackageManifest::from(PackageTestWithMetadata::default());

        p2.metadata.author = "NotCryptoGladi".to_string();

        assert_eq!(p1, p1);
        assert_eq!(p2, p2);
        assert_eq!(p1, p2);
        assert_eq!(p2, p1);
    }
}
