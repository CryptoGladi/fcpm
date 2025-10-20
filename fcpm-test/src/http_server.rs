use axum::Router;
use axum::routing::get;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::{LazyLock, Mutex, MutexGuard, PoisonError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HttpServerError {
    #[error("Error in run tokio runtime")]
    Tokio(#[from] tokio::io::Error),

    #[error("Mutex is poison")]
    Poison,
}

static USED_PORTS: LazyLock<UsedPorts> = LazyLock::new(UsedPorts::default);

#[derive(Debug, Clone)]
pub struct Port<'a> {
    port: u16,
    source: &'a UsedPorts,
}

impl<'a> Port<'a> {
    pub fn new(port: u16, source: &'a UsedPorts) -> Self {
        Self { port, source }
    }
}

impl<'a> Drop for Port<'a> {
    fn drop(&mut self) {
        self.source
            .remove(self.port)
            .expect("Remove port is not valid");
    }
}

impl<'a> PartialEq for Port<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.port == other.port
    }
}

impl<'a> Eq for Port<'a> {}

impl<'a> PartialOrd for Port<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.port.cmp(&other.port))
    }
}

impl<'a> std::fmt::Display for Port<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.port)
    }
}

type UsedPortsResult<'a, T> = Result<T, PoisonError<MutexGuard<'a, HashSet<u16>>>>;

#[derive(Debug, Default)]
pub struct UsedPorts {
    ports: Mutex<HashSet<u16>>,
}

impl<'a> UsedPorts {
    pub fn free(&'a self) -> UsedPortsResult<'a, Rc<Port<'a>>> {
        log::debug!("Get free port");

        let mut ports = self.ports.lock()?;

        for port in 1000..9999 {
            if ports.contains(&port) || !openport::is_free(port) {
                continue;
            }

            ports.insert(port);
            return Ok(Rc::new(Port { port, source: self }));
        }

        panic!("Not found free port");
    }

    pub fn used(&self) -> UsedPortsResult<'_, usize> {
        Ok(self.ports.lock()?.len())
    }

    pub fn remove(&self, port: u16) -> UsedPortsResult<'_, ()> {
        log::debug!("Delete port: {port}");

        self.ports.lock()?.remove(&port);
        Ok(())
    }
}

#[derive(Debug)]
pub struct HttpServerBuilder<'a> {
    root_text: String,
    used_ports: &'a UsedPorts,
}

impl<'a> Default for HttpServerBuilder<'a> {
    fn default() -> Self {
        Self {
            root_text: "It is root".to_string(),
            used_ports: &USED_PORTS,
        }
    }
}

impl<'a> HttpServerBuilder<'a> {
    pub fn new(used_ports: &'a UsedPorts) -> Self {
        Self {
            root_text: "".to_string(),
            used_ports,
        }
    }

    pub fn root_text(mut self, root_text: &str) -> Self {
        self.root_text = root_text.to_owned();

        self
    }

    pub fn used_ports(mut self, used_ports: &'a UsedPorts) -> Self {
        self.used_ports = used_ports;

        self
    }

    pub fn build(self) -> Result<HttpServer<'a>, HttpServerError> {
        log::debug!("Create http server");

        let app = Router::new().route("/", get(async || self.root_text));

        let runtime = tokio::runtime::Runtime::new()?;

        let port = self
            .used_ports
            .free()
            .map_err(|_| HttpServerError::Poison)?;

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
    port: Rc<Port<'a>>,
    handle: tokio::task::JoinHandle<()>,
    runtime: tokio::runtime::Runtime,
}

impl<'a> HttpServer<'a> {
    pub fn runtime(&self) -> &tokio::runtime::Runtime {
        &self.runtime
    }

    pub fn addr(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

impl<'a> Drop for HttpServer<'a> {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test]
    fn partial_eq_port() {
        let used_port = UsedPorts::default();

        let port1 = used_port.free().unwrap();
        let port2 = used_port.free().unwrap();

        assert_eq!(port1, port1);
        assert_eq!(port2, port2);
        assert_ne!(port1, port2);
    }

    #[test_log::test]
    fn partial_ord_port() {
        let used_port = UsedPorts::default();

        let port1 = Port::new(1000, &used_port);
        let port2 = Port::new(1001, &used_port);

        assert!(port1 < port2);
        assert!(port1 <= port2);

        assert!(port2 > port1);
        assert!(port2 >= port1);

        assert!(port1 >= port1);
        assert!(port2 >= port2);
        assert!(port1 <= port1);
        assert!(port2 <= port2);
    }

    #[test_log::test]
    fn impl_display_port() {
        let used_port = UsedPorts::default();
        let port = Port::new(123, &used_port);

        assert_eq!(format!("{port}"), "123");
    }

    #[test_log::test]
    fn get_free_port() {
        let used_port = UsedPorts::default();
        let port1 = used_port.free().unwrap();
        let port2 = used_port.free().unwrap();

        assert_ne!(port1, port2);
        assert_eq!(used_port.used().unwrap(), 2);

        drop(port1);
        drop(port2);

        assert_eq!(used_port.used().unwrap(), 0);
    }

    #[test_log::test]
    fn clone_port() {
        let used_port = UsedPorts::default();
        let port = used_port.free().unwrap();

        assert_eq!(used_port.used().unwrap(), 1);

        let port1 = port.clone();
        let port2 = port.clone();

        assert_eq!(used_port.used().unwrap(), 1);

        drop(port1);
        drop(port2);

        assert_eq!(used_port.used().unwrap(), 1);

        drop(port);
        assert_eq!(used_port.used().unwrap(), 0);
    }

    #[test_log::test]
    fn create_test_server() {
        let test_server = HttpServerBuilder::new(&USED_PORTS)
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
    fn default_http_server_builder() {
        let test_server = HttpServerBuilder::default().build().unwrap();

        let test = reqwest::blocking::get(test_server.addr())
            .unwrap()
            .text()
            .unwrap();

        assert_eq!(test, "It is root");
    }

    #[test_log::test]
    fn create_test_server_with_other_used_ports() {
        let used_ports = UsedPorts::default();

        let test_server = HttpServerBuilder::default()
            .used_ports(&used_ports)
            .build()
            .unwrap();

        let test = reqwest::blocking::get(test_server.addr())
            .unwrap()
            .text()
            .unwrap();
        assert_eq!(test, "It is root");
        assert_eq!(used_ports.used().unwrap(), 1); // Server used one port

        drop(test_server);
        assert_eq!(used_ports.used().unwrap(), 0);
    }
}
