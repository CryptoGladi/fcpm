use crate::PackageManagerCore;
use std::io::Read;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {}

pub trait PackageManagerStore: PackageManagerCore {
    fn install_from_reader(reader: &impl Read) {}

    fn install_from_url(url: impl AsRef<str>) {}

    fn delete(package_name: impl AsRef<str>) {}
}
