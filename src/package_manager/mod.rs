pub mod open;
pub mod repository;

use crate::package::Package;
use crate::package_manager::open::PackageManagerOpen;
use crate::package_manager::repository::PackageManagerRepository;
use repository::Repository;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::path::Path;

pub trait PackageManager<Metadata, P>:
    PackageManagerOpen<Metadata, P> + PackageManagerRepository<Metadata, P>
where
    Self: Sized,
    Metadata: Serialize + DeserializeOwned + Clone,
    P: Package<Metadata>,
{
    fn install_package_from_repository(&mut self, repository: Repository, name_package: &str);
    fn install_package_from_manifest(&mut self, path: impl AsRef<Path>);
    fn uninstall_package(&mut self, name_package: &str);
    fn upgrade_packages(&mut self);
}
