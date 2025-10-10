use serde::Serialize;
use serde::de::DeserializeOwned;
use std::borrow::Cow;

pub trait Package {
    type Metadata: Serialize + DeserializeOwned + Clone;

    fn new(
        name: String,
        version: String,
        repository_name: String,
        hashsum: String,
        metadata: Self::Metadata,
    ) -> Self;

    fn name(&self) -> Cow<'_, str>;

    // TODO semver?
    fn version(&self) -> Cow<'_, str>;

    fn repository_name(&self) -> Cow<'_, str>;

    fn hashsum(&self) -> Cow<'_, str>;

    fn metadata(&self) -> Cow<'_, Self::Metadata>;
}

#[cfg(test)]
pub(crate) mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(PartialEq, Eq, Debug)]
    pub(crate) struct PackageTest {
        pub name: String,
        pub version: String,
        pub repository_name: String,
        pub hashsum: String,
    }

    impl Default for PackageTest {
        fn default() -> Self {
            Self {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
                repository_name: "nixpkgs".to_string(),
                hashsum: "test-sha256".to_string(),
            }
        }
    }

    impl Package for PackageTest {
        type Metadata = ();

        fn new(
            name: String,
            version: String,
            repository_name: String,
            hashsum: String,
            _metadata: (),
        ) -> Self {
            Self {
                name,
                version,
                repository_name,
                hashsum,
            }
        }

        fn name(&self) -> Cow<'_, str> {
            Cow::Borrowed(&self.name)
        }

        fn version(&self) -> Cow<'_, str> {
            Cow::Borrowed(&self.version)
        }

        fn repository_name(&self) -> Cow<'_, str> {
            Cow::Borrowed(&self.repository_name)
        }

        fn hashsum(&self) -> Cow<'_, str> {
            Cow::Borrowed(&self.hashsum)
        }

        fn metadata(&self) -> Cow<'_, ()> {
            Cow::Owned(())
        }
    }

    #[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
    pub(crate) struct MetadataTest {
        pub created: String,
        pub author: String,
        pub stars: u32,
    }

    #[derive(PartialEq, Eq, Debug)]
    pub(crate) struct PackageTestWithMetadata {
        pub name: String,
        pub version: String,
        pub repository_name: String,
        pub hashsum: String,
        pub metadata: MetadataTest,
    }

    impl Default for PackageTestWithMetadata {
        fn default() -> Self {
            Self {
                name: "test-with-metadata".to_string(),
                version: "0.1.0".to_string(),
                repository_name: "nixpkgs".to_string(),
                hashsum: "test-sha256".to_string(),
                metadata: MetadataTest {
                    created: "12-09-2025".to_string(),
                    author: "CryptoGladi".to_string(),
                    stars: 100,
                },
            }
        }
    }

    impl Package for PackageTestWithMetadata {
        type Metadata = MetadataTest;

        fn new(
            name: String,
            version: String,
            repository_name: String,
            hashsum: String,
            metadata: MetadataTest,
        ) -> Self {
            Self {
                name,
                version,
                repository_name,
                hashsum,
                metadata,
            }
        }

        fn name(&self) -> Cow<'_, str> {
            Cow::Borrowed(&self.name)
        }

        fn version(&self) -> Cow<'_, str> {
            Cow::Borrowed(&self.version)
        }

        fn repository_name(&self) -> Cow<'_, str> {
            Cow::Borrowed(&self.repository_name)
        }

        fn hashsum(&self) -> Cow<'_, str> {
            Cow::Borrowed(&self.hashsum)
        }

        fn metadata(&self) -> Cow<'_, MetadataTest> {
            Cow::Borrowed(&self.metadata)
        }
    }
}
