pub mod open;
pub mod repository;

use crate::error::Error;
use repository::Repository;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::borrow::Cow;
use std::path::Path;

pub trait PackageManager<'a, Metadata>
where
    Metadata: Serialize + DeserializeOwned,
    Self: Sized,
{
    fn update_repositories(&mut self) -> Result<(), Error>; // TODO add progress 
    fn add_repository(&mut self, repository: Repository) -> Result<(), Error>;
    fn remove_repository(&mut self, name_repository: &str) -> Result<(), Error>;
    fn get_repositories(&self) -> Result<Cow<'a, Vec<Repository>>, Error>;

    fn install_package_from_repository(&mut self, repository: Repository, name_package: &str);
    fn install_package_from_manifest(&mut self, path: impl AsRef<Path>);
    fn uninstall_package(&mut self, name_package: &str);
    fn upgrade_packages(&mut self);
}
