#[cfg(any(not(feature = "http")))]
compile_error!("Not found method for downloading from network");

use super::RepositoryError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepositoryUrl {
    Http(String),
}

impl RepositoryUrl {
    pub fn download(&self) -> Result<String, RepositoryError> {
        #[cfg(feature = "logging")]
        log::debug!("Downloading repository manifest from: {:?}...", self);

        let manifest_text = match self {
            Self::Http(url) => reqwest::blocking::get(url)?.text()?,
        };

        Ok(manifest_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test]
    fn download_http() {
        let repository_url = RepositoryUrl::Http("https://example.com".to_string());
        let text = repository_url.download().unwrap();

        assert!(text.find("Example Domain").is_some());
    }

    #[test_log::test]
    #[should_panic]
    fn download_http_with_invalid_address() {
        let repository_url = RepositoryUrl::Http("https://example.c1".to_string());
        let text = repository_url.download().unwrap();

        assert!(text.find("Example Domain").is_some());
    }
}
