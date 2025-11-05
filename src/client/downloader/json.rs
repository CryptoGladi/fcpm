use super::{Downloader, DownloaderError};
use crate::client::downloader::reader::DownloaderReader;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloaderJson<'a, T>
where
    T: DeserializeOwned,
{
    url: &'a str,
    phantom: PhantomData<T>,
}

impl<'a, T> DownloaderJson<'a, T>
where
    T: DeserializeOwned,
{
    #[must_use]
    pub const fn new(url: &'a str) -> Self {
        Self {
            url,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Downloader<'a> for DownloaderJson<'a, T>
where
    T: DeserializeOwned,
{
    type Object = T;

    fn url(&self) -> &'a str {
        self.url
    }

    fn download(&self) -> Result<T, DownloaderError> {
        #[cfg(feature = "logging")]
        log::debug!("Downloading json from: {}...", self.url());

        let mut response = DownloaderReader::new(self.url).download()?;

        let mut json = String::new();
        response.read_to_string(&mut json)?;

        Ok(serde_json::from_str(&json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::example::package::MetadataExample;
    use fcpm_test::http_server::HttpServerBuilder;

    #[test_log::test]
    #[cfg(feature = "http")]
    fn download_http() {
        let json = vec![MetadataExample::default()];
        let json_str = serde_json::to_string(&json).unwrap();

        let test_server = HttpServerBuilder::default()
            .root_text(json_str)
            .build()
            .unwrap();

        let addr = test_server.addr();
        let downloader = DownloaderJson::new(&addr);
        let gotten_json: Vec<MetadataExample> = downloader.download().unwrap();

        assert_eq!(gotten_json, json);
    }

    #[test_log::test]
    #[cfg(feature = "http")]
    fn download_http_with_empty_data() {
        let json = ();
        let json_str = serde_json::to_string(&json).unwrap();

        let test_server = HttpServerBuilder::default()
            .root_text(json_str)
            .build()
            .unwrap();

        let addr = test_server.addr();
        let downloader = DownloaderJson::new(&addr);

        let gotten_json: () = downloader.download().unwrap();

        assert_eq!(gotten_json, json);
    }

    #[test_log::test]
    #[cfg(feature = "http")]
    #[should_panic]
    fn download_http_with_invalid_address() {
        let downloader = DownloaderJson::<()>::new("https://lol.kek.sa");
        let _text = downloader.download().unwrap();
    }

    #[test_log::test]
    #[cfg(feature = "http")]
    #[should_panic]
    fn download_http_with_empty_address() {
        let downloader = DownloaderJson::<()>::new("");
        let _text = downloader.download().unwrap();
    }
}
