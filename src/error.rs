use crate::package_manager::{open::OpenError, repository::RepositoryError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error in opening: {0}")]
    Open(#[from] OpenError),

    #[error("Error in repository: {0}")]
    Repository(#[from] RepositoryError),
}
