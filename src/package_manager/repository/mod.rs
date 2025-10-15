pub mod downloader;
pub mod repository_manifest;

use crate::{package::Package, package_manager::open::PackageManagerOpen};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Already have repository with same name")]
    AlreadyHave,

    #[error("Error IO")]
    IO(#[from] std::io::Error),

    #[error("Link: `{0}` is invalid or not support")]
    LinkInvalid(String),

    #[cfg(feature = "http")]
    #[error("Error reqwest (http)")]
    Reqwest(#[from] reqwest::Error),

    #[error("Json error")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repository {
    name: String,
    url: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repositories(Vec<Repository>);

impl Repositories {
    pub fn add(&mut self, repository: Repository) -> Result<(), RepositoryError> {
        #[cfg(feature = "logging")]
        log::debug!("Add to repositories: {:?}", repository);

        if self.get(&repository.name).is_some() {
            return Err(RepositoryError::AlreadyHave);
        }

        self.0.push(repository);
        Ok(())
    }

    pub fn get(&self, repository_name: &str) -> Option<&Repository> {
        #[cfg(feature = "logging")]
        log::debug!("Get repository by name: {repository_name}");

        self.0
            .iter()
            .find(|&repository| repository.name == repository_name)
    }
}

impl std::ops::Index<&str> for Repositories {
    type Output = Repository;

    fn index(&self, index: &str) -> &Self::Output {
        self.get(index).expect("Repository not found by name")
    }
}

fn get_path<P>(package_manager: &impl PackageManagerOpen<P>) -> PathBuf
where
    P: Package,
{
    let options = package_manager.get_options();
    options.path_repository(package_manager.path())
}

pub trait PackageManagerRepository<P>: PackageManagerOpen<P>
where
    Self: Sized,
    P: Package,
{
    fn update_repositories(&mut self) -> Result<(), RepositoryError> {
        Ok(())
    }

    fn add_repository(&mut self, repository: Repository) -> Result<(), RepositoryError>;
    fn remove_repository(&mut self, name_repository: &str) -> Result<(), RepositoryError>;
    fn get_repositories(&self) -> Result<Cow<'_, Vec<Repository>>, RepositoryError> {
        let _path = get_path(self);

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use super::*;

    static REPOSITORY_TEST: LazyLock<Repository> = LazyLock::new(|| Repository {
        name: "sa".to_string(),
        url: "dsd".to_string(),
    });

    #[test_log::test]
    fn repositories_add() {
        let mut repositories = Repositories::default();
        repositories.add(REPOSITORY_TEST.clone()).unwrap();
    }

    #[test_log::test]
    #[should_panic(expected = "PANIC: AlreadyHave")]
    fn repositories_add_with_same_name() {
        let mut repositories = Repositories::default();

        repositories.add(REPOSITORY_TEST.clone()).unwrap();
        repositories.add(REPOSITORY_TEST.clone()).expect("PANIC");
    }

    #[test_log::test]
    fn repositories_get() {
        let mut repositories = Repositories::default();
        repositories.add(REPOSITORY_TEST.clone()).unwrap();

        let repository = repositories.get(&REPOSITORY_TEST.name).unwrap();
        assert_eq!(*repository, *REPOSITORY_TEST);
    }

    #[test_log::test]
    #[should_panic(expected = "package not found")]
    fn repositories_get_not_found() {
        let repositories = Repositories::default();

        repositories.get("package-test").expect("package not found");
    }

    #[test_log::test]
    fn repositories_index() {
        let mut repositories = Repositories::default();
        repositories.add(REPOSITORY_TEST.clone()).unwrap();

        let repository = &repositories[&REPOSITORY_TEST.name];
        assert_eq!(*repository, *REPOSITORY_TEST);
    }

    #[test_log::test]
    #[should_panic(expected = "Repository not found by name")]
    fn repositories_index_not_found() {
        let repositories = Repositories::default();

        // PANIC
        let _repository = &repositories["package-test"];
    }
}
