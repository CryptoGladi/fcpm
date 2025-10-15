#[allow(clippy::non_minimal_cfg, reason = "For feature")]
#[cfg(any(not(feature = "http")))]
compile_error!("Not found method for downloading from network");

use super::RepositoryError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
enum DownloaderType {
    #[cfg(feature = "http")]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Downloader<'a> {
    url: &'a str,
}

impl<'a> Downloader<'a> {
    pub fn new(url: &'a str) -> Self {
        Self { url }
    }

    pub fn download(&self) -> Result<String, RepositoryError> {
        #[cfg(feature = "logging")]
        log::debug!("Downloading repository manifest from: {:?}...", self.url);

        let strategy = DownloaderType::get(self.url)
            .ok_or(RepositoryError::LinkInvalid(self.url.to_string()))?;

        let manifest_str = match strategy {
            #[cfg(feature = "http")]
            DownloaderType::Http => {
                use reqwest::blocking::ClientBuilder;

                let timeout = match cfg!(test) {
                    true => Duration::from_secs(2),
                    false => Duration::from_secs(10),
                };

                let client = ClientBuilder::default().timeout(timeout).build()?;
                client.get(self.url).send()?.text()?
            }
        };

        Ok(manifest_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn download_http() {
        let downloader = Downloader::new("https://example.com");
        let text = downloader.download().unwrap();

        assert!(text.find("Example Domain").is_some());
    }

    #[test_log::test]
    #[cfg(feature = "http")]
    #[should_panic]
    fn download_http_with_invalid_address() {
        let downloader = Downloader::new("https://example.c1");
        let text = downloader.download().unwrap();

        assert!(text.find("Example Domain").is_some());
    }
}
