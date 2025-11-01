//#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(clippy::perf)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::panic)]
#![warn(clippy::needless_pass_by_value)]
#![warn(clippy::unreadable_literal)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::as_conversions)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod client;
pub mod error;
pub mod example;
pub mod package;
pub mod server;

pub use client::PackageManager;
pub use client::core::PackageManagerCore;
pub use client::core::index::Index;
pub use client::core::lockfile::LockFile;
pub use client::core::options::OpenOptions;
pub use package::Package;
