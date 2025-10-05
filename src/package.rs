use serde::Serialize;
use serde::de::DeserializeOwned;
use std::borrow::Cow;

pub trait Package<Metadata>
where
    Metadata: Serialize + DeserializeOwned + Clone,
{
    fn name(&self) -> Cow<'_, str>;

    // TODO semver?
    fn version(&self) -> Cow<'_, str>;

    fn repository_name(&self) -> Cow<'_, str>;

    fn hashsum(&self) -> Cow<'_, str>;

    fn metadata(&self) -> Cow<'_, Metadata>;
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

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

    impl Package<()> for PackageTest {
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
}
