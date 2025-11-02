use crate::Package;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PackageExample {
    pub name: String,
    pub version: Version,
    pub repository_name: String,
}

impl Default for PackageExample {
    fn default() -> Self {
        Self {
            name: "test".to_string(),
            version: Version::new(0, 1, 0),
            repository_name: "nixpkgs".to_string(),
        }
    }
}

impl Package for PackageExample {
    type Metadata = ();

    fn new(
        name: String,
        version: Version,
        repository_name: String,
        _metadata: Self::Metadata,
    ) -> Self {
        Self {
            name,
            version,
            repository_name,
        }
    }

    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.name)
    }

    fn version(&self) -> Cow<'_, Version> {
        Cow::Borrowed(&self.version)
    }

    fn repository_name(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.repository_name)
    }

    fn metadata(&self) -> Cow<'_, ()> {
        Cow::Owned(())
    }
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct MetadataExample {
    pub created: String,
    pub author: String,
    pub stars: u32,
}

#[derive(PartialEq, Eq, Debug)]
pub struct PackageExampleWithMetadata {
    pub name: String,
    pub version: Version,
    pub repository_name: String,
    pub metadata: MetadataExample,
}

impl Default for PackageExampleWithMetadata {
    fn default() -> Self {
        Self {
            name: "test-with-metadata".to_string(),
            version: Version::new(1, 0, 0),
            repository_name: "nixpkgs".to_string(),
            metadata: MetadataExample {
                created: "12-09-2025".to_string(),
                author: "CryptoGladi".to_string(),
                stars: 100,
            },
        }
    }
}

impl Package for PackageExampleWithMetadata {
    type Metadata = MetadataExample;

    fn new(
        name: String,
        version: Version,
        repository_name: String,
        metadata: Self::Metadata,
    ) -> Self {
        Self {
            name,
            version,
            repository_name,
            metadata,
        }
    }

    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.name)
    }

    fn version(&self) -> Cow<'_, Version> {
        Cow::Borrowed(&self.version)
    }

    fn repository_name(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.repository_name)
    }

    fn metadata(&self) -> Cow<'_, MetadataExample> {
        Cow::Borrowed(&self.metadata)
    }
}
