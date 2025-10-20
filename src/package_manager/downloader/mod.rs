#[cfg(not(feature = "http"))]
compile_error!("Not found method for downloading from network");

pub mod file;
pub mod json;

use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DownloaderError {
    #[error("IO Error")]
    IO(#[from] std::io::Error),

    #[error("Link: `{0}` is invalid or not support")]
    LinkInvalid(String),

    #[cfg(feature = "http")]
    #[cfg_attr(docsrs, doc(cfg(feature = "http")))]
    #[error("Error reqwest (http)")]
    Reqwest(#[from] reqwest::Error),

    #[error("Json error")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloaderType {
    #[cfg(feature = "http")]
    #[cfg_attr(docsrs, doc(cfg(feature = "http")))]
    Http,
}

impl DownloaderType {
    fn get(url: &str) -> Option<Self> {
        #[cfg(feature = "logging")]
        log::trace!("Get downloader type for {url}");

        let mut split = url.split("://");

        match split.next() {
            #[cfg(feature = "http")]
            Some("http") | Some("https") => Some(Self::Http),
            _ => None,
        }
    }
}

pub trait Downloader<'a> {
    type Object;

    fn url(&self) -> &'a str;

    fn url_type(&self) -> Result<DownloaderType, DownloaderError> {
        match DownloaderType::get(self.url()) {
            Some(url_type) => Ok(url_type),
            None => Err(DownloaderError::LinkInvalid(self.url().to_string())),
        }
    }

    fn timeout(&self) -> Duration {
        match cfg!(test) {
            true => Duration::from_secs(1),
            false => Duration::from_secs(10),
        }
    }

    fn download(&self) -> Result<Self::Object, DownloaderError>;
}

pub fn download<'a, T: Downloader<'a>>(downloader: T) -> Result<T::Object, DownloaderError> {
    downloader.download()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fcpm_test::http_server::HttpServerBuilder;
    use serde::Deserialize;

    #[test_log::test]
    fn downloader_empty_string() {
        assert!(matches!(DownloaderType::get(""), None));
    }

    #[test_log::test]
    fn downloader_type_not_found() {
        assert!(matches!(DownloaderType::get("wtf"), None));
    }

    #[test_log::test]
    #[cfg(feature = "http")]
    fn downloader_type_http() {
        assert_eq!(
            DownloaderType::get("https://example.com").unwrap(),
            DownloaderType::Http
        );

        assert_eq!(
            DownloaderType::get("http://example.com").unwrap(),
            DownloaderType::Http
        );
    }

    #[test_log::test]
    #[cfg(feature = "http")]
    fn simple_download() {
        use crate::package_manager::downloader::json::DownloaderJson;

        let http_server = HttpServerBuilder::default()
            .root_text(r#"{"data": "w"}"#)
            .build()
            .unwrap();

        #[derive(Deserialize)]
        struct T {
            data: String,
        }

        let json: T = super::download(DownloaderJson::new(&http_server.addr())).unwrap();
        assert_eq!(json.data, "w");
    }
}
