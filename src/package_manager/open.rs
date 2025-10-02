use thiserror::Error;

pub struct OpenOptions {}

#[derive(Debug, Error)]
pub enum OpenError {
    #[error("Storage not found")]
    StorageNotFound,
}

fn open() {}
