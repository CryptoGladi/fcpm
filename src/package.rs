use std::borrow::Cow;

use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait Package<'a, Metadata>
where
    Metadata: Serialize + DeserializeOwned,
{
    fn name() -> Cow<'a, str>;

    // TODO semver?
    fn version() -> Cow<'a, str>;
}
