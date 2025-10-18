use std::collections::HashSet;
use std::sync::{LazyLock, Mutex};

static USED_PORTS: LazyLock<UsedPorts> = LazyLock::new(UsedPorts::default);

#[derive(Debug, Default)]
pub struct UsedPorts {
    ports: Mutex<HashSet<u16>>,
}

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
        self.source.remove(self.port);
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

impl UsedPorts {
    pub fn free(&self) -> Port<'_> {
        log::debug!("Get free port");

        let mut ports = self.ports.lock().unwrap();

        for port in 1000..9999 {
            if ports.contains(&port) || !openport::is_free(port) {
                continue;
            }

            ports.insert(port);
            return Port { port, source: self };
        }

        panic!("Not found free port");
    }

    pub fn used(&self) -> usize {
        self.ports.lock().unwrap().len()
    }

    pub fn remove(&self, port: u16) {
        log::debug!("Delete port: {port}");

        self.ports.lock().unwrap().remove(&port);
    }
}

pub struct HttpServer<'a> {
    port: Port<'a>,
    handle: tokio::task::JoinHandle<()>,
    runtime: tokio::runtime::Runtime,
}

impl<'a> HttpServer<'a> {
    pub fn new(root_text: String) -> Self {
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

        let port1 = used_port.free();
        let port2 = used_port.free();

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
        let port1 = used_port.free();
        let port2 = used_port.free();

        assert_ne!(port1, port2);
        assert_eq!(used_port.used(), 2);

        drop(port1);
        drop(port2);

        assert_eq!(used_port.used(), 0);
    }

    #[test_log::test]
    fn create_test_server() {
        let test_server = HttpServer::new("Test is done!".to_string());

        let test = reqwest::blocking::get(test_server.addr())
            .unwrap()
            .text()
            .unwrap();

        assert_eq!(test, "Test is done!");
    }
}
