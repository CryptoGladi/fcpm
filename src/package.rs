use semver::Version;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::borrow::Cow;

pub trait Package {
    type Metadata: Serialize + DeserializeOwned + Clone;

    fn new(
        name: String,
        version: Version,
        repository_name: String,
        metadata: Self::Metadata,
    ) -> Self;

    fn name(&self) -> Cow<'_, str>;

    fn version(&self) -> Cow<'_, Version>;

    fn repository_name(&self) -> Cow<'_, str>;

    fn metadata(&self) -> Cow<'_, Self::Metadata>;
}
