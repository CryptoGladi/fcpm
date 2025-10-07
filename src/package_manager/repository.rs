use crate::{package::Package, package_manager::open::PackageManagerOpen};
use serde::{Serialize, de::DeserializeOwned};
use std::borrow::Cow;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RepositoryUrl {
    Http(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repository {
    name: String,
    url: RepositoryUrl,
}

pub trait PackageManagerRepository<Metadata, P>: PackageManagerOpen<Metadata, P>
where
    Self: Sized,
    Metadata: Serialize + DeserializeOwned + Clone,
    P: Package<Metadata>,
{
    fn update_repositories(&mut self) -> Result<(), RepositoryError> {
        let options = self.get_options();

        Ok(())
    }

    fn add_repository(&mut self, repository: Repository) -> Result<(), RepositoryError>;
    fn remove_repository(&mut self, name_repository: &str) -> Result<(), RepositoryError>;
    fn get_repositories(&self) -> Result<Cow<'_, Vec<Repository>>, RepositoryError>;
}
