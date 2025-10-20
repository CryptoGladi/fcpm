use std::{
    collections::HashSet,
    ops::Range,
    sync::{LazyLock, Mutex, MutexGuard, PoisonError},
};

pub static USED_PORTS: LazyLock<UsedPorts> = LazyLock::new(UsedPorts::default);
const RANGE_PORTS: Range<u16> = 1000..9999;

#[derive(Debug)]
pub struct Port<'a> {
    port: u16,
    source: &'a UsedPorts,
}

impl<'a> Port<'a> {
    pub fn new(port: u16, source: &'a UsedPorts) -> Self {
        Self { port, source }
    }

    pub fn port(&self) -> u16 {
        self.port
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

impl<'a> From<Port<'a>> for u16 {
    fn from(port: Port<'a>) -> Self {
        port.port
    }
}

pub(crate) type UsedPortsResult<'a, T> = Result<T, PoisonError<MutexGuard<'a, HashSet<u16>>>>;

#[derive(Debug, Default)]
pub struct UsedPorts {
    ports: Mutex<HashSet<u16>>,
}

impl<'a> UsedPorts {
    pub fn free(&'a self) -> UsedPortsResult<'a, Port<'a>> {
        log::debug!("Get free port");

        let mut ports = self.ports.lock()?;

        for port in RANGE_PORTS {
            if ports.contains(&port) || !openport::is_free(port) {
                continue;
            }

            ports.insert(port);
            return Ok(Port::new(port, self));
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

    pub fn insert(&self, port: u16) -> UsedPortsResult<'_, bool> {
        Ok(self.ports.lock()?.insert(port))
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;

    #[test_log::test]
    fn partial_eq() {
        let used_port = UsedPorts::default();

        let port1 = used_port.free().unwrap();
        let port2 = used_port.free().unwrap();

        assert_eq!(port1, port1);
        assert_eq!(port2, port2);
        assert_ne!(port1, port2);
    }

    #[test_log::test]
    fn partial_ord() {
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
    fn impl_display() {
        let used_port = UsedPorts::default();
        let port = Port::new(123, &used_port);

        assert_eq!(format!("{port}"), "123");
    }

    #[test_log::test]
    fn free() {
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
    fn clone() {
        let used_port = UsedPorts::default();
        let port = used_port.free().unwrap();
        let port = Rc::new(port);

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
    fn get_port() {
        let port = Port::new(5462, &USED_PORTS);

        assert_eq!(port.port(), 5462);
    }

    #[test_log::test]
    fn insert() {
        let used_ports = UsedPorts::default();

        used_ports.insert(1000).unwrap();
        assert_eq!(used_ports.used().unwrap(), 1);
    }
}
