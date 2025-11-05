use super::{Downloader, DownloaderError, DownloaderType};
use std::io::Read;

pub struct DownloaderReader<'a> {
    url: &'a str,
}

impl<'a> DownloaderReader<'a> {
    #[must_use]
    pub const fn new(url: &'a str) -> Self {
        Self { url }
    }
}

impl<'a> Downloader<'a> for DownloaderReader<'a> {
    type Object = Box<dyn Read>;

    fn url(&self) -> &'a str {
        self.url
    }

    fn download(&self) -> Result<Self::Object, DownloaderError> {
        #[cfg(feature = "logging")]
        log::debug!("Downloading reader from: `{}`...", self.url(),);

        let strategy = self.url_type()?;

        let reader = match strategy {
            #[cfg(feature = "http")]
            DownloaderType::Http => {
                use reqwest::blocking::ClientBuilder;

                let timeout = self.timeout();
                let client = ClientBuilder::default().timeout(timeout).build()?;
                let bytes = client.get(self.url).send()?;

                Box::new(bytes)
            }
        };

        Ok(reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fcpm_test::http_server::HttpServerBuilder;
    use std::io::Read;

    #[test_log::test]
    #[cfg(feature = "http")]
    fn download_http() {
        let test_server = HttpServerBuilder::default()
            .root_text("SUPER OMEGA INFORMATION".to_string())
            .build()
            .unwrap();

        let addr = test_server.addr();
        let downloader = DownloaderReader::new(&addr);
        let mut bufreader = downloader.download().unwrap();

        let mut result = String::new();
        bufreader.read_to_string(&mut result).unwrap();

        assert_eq!(result, "SUPER OMEGA INFORMATION");
    }

    #[test_log::test]
    #[cfg(feature = "http")]
    fn download_http_with_empty_data() {
        let test_server = HttpServerBuilder::default()
            .root_text("".to_string())
            .build()
            .unwrap();

        let addr = test_server.addr();
        let downloader = DownloaderReader::new(&addr);
        let mut bufreader = downloader.download().unwrap();

        let mut result = String::new();
        bufreader.read_to_string(&mut result).unwrap();

        assert!(result.is_empty());
    }

    #[test_log::test]
    #[cfg(feature = "http")]
    #[should_panic]
    fn download_http_with_invalid_address() {
        let downloader = DownloaderReader::new("https://lol.kek.sa");
        let _text = downloader.download().unwrap();
    }

    #[test_log::test]
    #[cfg(feature = "http")]
    #[should_panic]
    fn download_http_with_empty_address() {
        let downloader = DownloaderReader::new("");
        let _text = downloader.download().unwrap();
    }
}
