use crate::Package;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct PackageExample {
    pub name: String,
    pub version: String,
    pub repository_name: String,
    pub hashsum: String,
}

impl Default for PackageExample {
    fn default() -> Self {
        Self {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
            repository_name: "nixpkgs".to_string(),
            hashsum: "test-sha256".to_string(),
        }
    }
}

impl Package for PackageExample {
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
pub struct MetadataExample {
    pub created: String,
    pub author: String,
    pub stars: u32,
}

#[derive(PartialEq, Eq, Debug)]
pub struct PackageExampleWithMetadata {
    pub name: String,
    pub version: String,
    pub repository_name: String,
    pub hashsum: String,
    pub metadata: MetadataExample,
}

impl Default for PackageExampleWithMetadata {
    fn default() -> Self {
        Self {
            name: "test-with-metadata".to_string(),
            version: "0.1.0".to_string(),
            repository_name: "nixpkgs".to_string(),
            hashsum: "test-sha256".to_string(),
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
        version: String,
        repository_name: String,
        hashsum: String,
        metadata: MetadataExample,
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

    fn metadata(&self) -> Cow<'_, MetadataExample> {
        Cow::Borrowed(&self.metadata)
    }
}
