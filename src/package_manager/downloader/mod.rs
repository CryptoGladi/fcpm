#[cfg(not(feature = "http"))]
compile_error!("Not found method for downloading from network");

pub mod file;
pub mod json;

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DownloaderError {
    #[error("IO Error")]
    IO(#[from] std::io::Error),

    #[error("Link: `{0}` is invalid or not support")]
    LinkInvalid(String),

    #[cfg(feature = "http")]
    #[error("Error reqwest (http)")]
    Reqwest(#[from] reqwest::Error),

    #[error("Json error")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloaderType {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloaderFile<'a> {
    url: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use std::collections::HashSet;
    use std::sync::{LazyLock, Mutex};

    static USED_PORTS: LazyLock<UsedPorts> = LazyLock::new(|| UsedPorts::default());

    #[derive(Debug, Default)]
    pub(crate) struct UsedPorts {
        ports: Mutex<HashSet<u16>>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub(crate) struct Port(u16);

    impl Drop for Port {
        fn drop(&mut self) {
            USED_PORTS.remove(self.0);
        }
    }

    impl std::fmt::Display for Port {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl UsedPorts {
        pub(crate) fn free(&self) -> Port {
            #[cfg(feature = "logging")]
            log::debug!("Get free port");

            let mut ports = self.ports.lock().unwrap();

            if ports.len() >= 9999 - 1000 {
                panic!("Many used ports");
            }

            let mut rng = rand::rng();

            let port = loop {
                let port = rng.random_range(1000..=9999);

                if !openport::is_free(port) || ports.contains(&port) {
                    continue;
                }

                break port;
            };

            ports.insert(port);
            Port(port)
        }

        fn used(&self) -> usize {
            self.ports.lock().unwrap().len()
        }

        fn remove(&self, port: u16) {
            #[cfg(feature = "logging")]
            log::debug!("Delete port: {port}");

            self.ports.lock().unwrap().remove(&port);
        }
    }

    pub(crate) struct TestHttpServer {
        port: Port,
        handle: tokio::task::JoinHandle<()>,
        runtime: tokio::runtime::Runtime,
    }

    impl TestHttpServer {
        pub(crate) fn create(root_text: String) -> Self {
            #[cfg(feature = "logging")]
            log::debug!("Create test server");

            use axum::Router;
            use axum::routing::get;

            let port = USED_PORTS.free();
            let root_text_clone = root_text.clone();
            let app = Router::new().route("/", get(async || -> String { root_text_clone }));

            let runtime = tokio::runtime::Runtime::new().unwrap();

            let addr = format!("127.0.0.1:{port}");
            let handle = runtime.spawn(async move {
                let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
                axum::serve(listener, app).await.unwrap();
            });

            Self {
                port,
                handle,
                runtime,
            }
        }

        #[allow(unused)]
        pub(crate) fn runtime(&self) -> &tokio::runtime::Runtime {
            &self.runtime
        }

        pub(crate) fn addr(&self) -> String {
            format!("http://127.0.0.1:{}", self.port)
        }
    }

    impl Drop for TestHttpServer {
        fn drop(&mut self) {
            self.handle.abort();
        }
    }

    #[test_log::test]
    fn impl_display_port() {
        let port = Port(123);

        assert_eq!(format!("{port}"), "123");
    }

    #[test_log::test]
    fn get_free_port() {
        let port1 = USED_PORTS.free();
        let port2 = USED_PORTS.free();

        assert_ne!(port1, port2);
        assert!(USED_PORTS.used() >= 2);

        drop(port1);
        drop(port2);
    }

    #[test_log::test]
    fn create_test_server() {
        let test_server = TestHttpServer::create("Test is done!".to_string());

        let test = reqwest::blocking::get(test_server.addr())
            .unwrap()
            .text()
            .unwrap();

        assert_eq!(test, "Test is done!");
    }

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
}
