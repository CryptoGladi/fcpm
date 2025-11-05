use super::{Downloader, DownloaderError};
use crate::client::downloader::reader::DownloaderReader;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

pub struct DownloaderFile<'a> {
    url: &'a str,
    path: PathBuf,
}

impl<'a> DownloaderFile<'a> {
    pub fn new(url: &'a str, path: impl AsRef<Path>) -> Self {
        Self {
            url,
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl<'a> Downloader<'a> for DownloaderFile<'a> {
    type Object = ();

    fn url(&self) -> &'a str {
        self.url
    }

    fn download(&self) -> Result<(), DownloaderError> {
        #[cfg(feature = "logging")]
        log::debug!(
            "Downloading file from: `{}` to `{}`...",
            self.url(),
            self.path.display()
        );

        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&self.path)?;

        let mut response = DownloaderReader::new(self.url).download()?;
        std::io::copy(&mut response, &mut file)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fcpm_test::http_server::HttpServerBuilder;
    use tempfile::tempdir;

    #[test_log::test]
    #[cfg(feature = "http")]
    fn download_http() {
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("test_file.txt");

        let test_server = HttpServerBuilder::default()
            .root_text("SUPER OMEGA INFORMATION".to_string())
            .build()
            .unwrap();

        let addr = test_server.addr();
        let downloader = DownloaderFile::new(&addr, &path);
        downloader.download().unwrap();

        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "SUPER OMEGA INFORMATION"
        );
    }

    #[test_log::test]
    #[cfg(feature = "http")]
    fn download_http_with_empty_data() {
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("test_file.txt");
        let test_server = HttpServerBuilder::default()
            .root_text("".to_string())
            .build()
            .unwrap();

        let addr = test_server.addr();
        let downloader = DownloaderFile::new(&addr, &path);
        downloader.download().unwrap();

        assert!(std::fs::read_to_string(&path).unwrap().is_empty());
    }

    #[test_log::test]
    #[cfg(feature = "http")]
    #[should_panic]
    fn download_http_with_empty_path() {
        let test_server = HttpServerBuilder::default()
            .root_text("SUPER OMEGA INFORMATION".to_string())
            .build()
            .unwrap();

        let addr = test_server.addr();
        let downloader = DownloaderFile::new(&addr, "");
        downloader.download().unwrap();
    }

    #[test_log::test]
    #[cfg(feature = "http")]
    #[should_panic]
    fn download_http_with_invalid_address() {
        let downloader = DownloaderFile::new("https://lol.kek.sa", "");
        let _text = downloader.download().unwrap();
    }

    #[test_log::test]
    #[cfg(feature = "http")]
    #[should_panic]
    fn download_http_with_empty_address() {
        let downloader = DownloaderFile::new("", "");
        let _text = downloader.download().unwrap();
    }
}
