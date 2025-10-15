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
