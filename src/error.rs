use crate::package_manager::open::OpenError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error in opening: {0}")]
    Open(#[from] OpenError),
}
