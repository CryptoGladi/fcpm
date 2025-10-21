use crate::port::{Port, RANGE_PORTS, USED_PORTS, UsedPorts};
use axum::Router;
use axum::routing::get;
use std::ops::Range;
use thiserror::Error;

const ROOT_TEXT_DEFAULT: &str = "It is root";

#[derive(Error, Debug)]
pub enum HttpServerError {
    #[error("Error in run tokio runtime")]
    Tokio(#[from] tokio::io::Error),

    #[error("Mutex is poison")]
    Poison,

    #[error("Port {0} is busy")]
    PortIsBusy(u16),
}

#[derive(Debug)]
pub struct HttpServerBuilder<'a> {
    root_text: String,
    strict_port: Option<u16>,
    range_port: Range<u16>,
    used_ports: &'a UsedPorts,
}

impl Default for HttpServerBuilder<'_> {
    fn default() -> Self {
        Self {
            root_text: ROOT_TEXT_DEFAULT.to_string(),
            strict_port: None,
            range_port: RANGE_PORTS,
            used_ports: &USED_PORTS,
        }
    }
}

impl<'a> HttpServerBuilder<'a> {
    pub const fn new(used_ports: &'a UsedPorts, range_port: Range<u16>) -> Self {
        Self {
            root_text: String::new(),
            strict_port: None,
            range_port,
            used_ports,
        }
    }

    #[must_use]
    pub fn root_text(mut self, root_text: &str) -> Self {
        root_text.clone_into(&mut self.root_text);

        self
    }

    #[must_use]
    pub const fn strict_port(mut self, strict_port: Option<u16>) -> Self {
        self.strict_port = strict_port;

        self
    }

    #[must_use]
    pub const fn range_port(mut self, range_port: Range<u16>) -> Self {
        self.range_port = range_port;

        self
    }

    #[must_use]
    pub const fn used_ports(mut self, used_ports: &'a UsedPorts) -> Self {
        self.used_ports = used_ports;

        self
    }

    pub fn build(self) -> Result<HttpServer<'a>, HttpServerError> {
        log::debug!("Create http server");

        let app = Router::new().route("/", get(async || self.root_text));

        let runtime = tokio::runtime::Runtime::new()?;

        let port = match self.strict_port {
            None => self
                .used_ports
                .free_by_range(self.range_port)
                .map_err(|_| HttpServerError::Poison)?,
            Some(port) => {
                let already_have = !self
                    .used_ports
                    .insert(port)
                    .map_err(|_| HttpServerError::Poison)?;

                if already_have || !openport::is_free(port) {
                    return Err(HttpServerError::PortIsBusy(port));
                }

                Port::new(port, self.used_ports)
            }
        };

        let addr = format!("127.0.0.1:{port}");
        let handle = runtime.spawn(async move {
            let listener = tokio::net::TcpListener::bind(&addr)
                .await
                .expect("Bind error");
            axum::serve(listener, app)
                .await
                .expect("Start server error");
        });

        Ok(HttpServer {
            port,
            handle,
            runtime,
        })
    }
}

pub struct HttpServer<'a> {
    port: Port<'a>,
    handle: tokio::task::JoinHandle<()>,
    runtime: tokio::runtime::Runtime,
}

impl<'a> HttpServer<'a> {
    pub const fn runtime(&self) -> &tokio::runtime::Runtime {
        &self.runtime
    }

    pub fn addr(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    pub const fn port(&'a self) -> &'a Port<'a> {
        &self.port
    }
}

impl Drop for HttpServer<'_> {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test]
    fn create() {
        let test_server = HttpServerBuilder::new(&USED_PORTS, RANGE_PORTS)
            .root_text("Test is done!")
            .build()
            .unwrap();

        let test = reqwest::blocking::get(test_server.addr())
            .unwrap()
            .text()
            .unwrap();

        assert_eq!(test, "Test is done!");
    }

    #[test_log::test]
    fn default() {
        let test_server = HttpServerBuilder::default().build().unwrap();

        let test = reqwest::blocking::get(test_server.addr())
            .unwrap()
            .text()
            .unwrap();

        assert_eq!(test, ROOT_TEXT_DEFAULT);
    }

    #[test_log::test]
    fn create_with_other_used_ports() {
        let used_ports = UsedPorts::default();
        let test_server = HttpServerBuilder::default()
            .used_ports(&used_ports)
            .range_port(2000..4000) // To avoid conflicts
            .build()
            .unwrap();

        let test = reqwest::blocking::get(test_server.addr())
            .unwrap()
            .text()
            .unwrap();

        assert_eq!(test, ROOT_TEXT_DEFAULT);
        assert_eq!(used_ports.used().unwrap(), 1); // Server used one port

        drop(test_server);
        assert_eq!(used_ports.used().unwrap(), 0);
    }

    #[test_log::test]
    fn strict_port() {
        let test_server = HttpServerBuilder::default()
            .strict_port(Some(9995))
            .build()
            .unwrap();

        assert_eq!(test_server.port(), 9995);
    }

    #[test_log::test]
    fn range_port() {
        let test_server = HttpServerBuilder::default()
            .range_port(5000..9000)
            .build()
            .unwrap();

        assert!(*test_server.port() >= 5000);
    }
}
