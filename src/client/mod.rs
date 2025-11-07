pub mod core;
pub mod downloader;
pub mod repository;
pub mod store;

use crate::client::core::PackageManagerCore;
use crate::client::repository::PackageManagerRepository;
use crate::package::Package;

pub trait PackageManager<P>: PackageManagerCore + PackageManagerRepository
where
    Self: Sized,
    P: Package,
{
    //fn install_package_from_repository(&mut self, repository: Repository, name_package: &str);
    //fn install_package_from_manifest(&mut self, path: impl AsRef<Path>);
    //fn uninstall_package(&mut self, name_package: &str);
    //fn upgrade_packages(&mut self)
}
