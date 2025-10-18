#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod error;
pub mod example;
pub mod package;
pub mod package_manager;
pub mod server;

pub use package::Package;
pub use package_manager::PackageManager;
pub use package_manager::open::PackageManagerOpen;
pub use package_manager::open::index::Index;
pub use package_manager::open::lockfile::LockFile;
pub use package_manager::open::options::OpenOptions;
