use std::{
    collections::HashSet,
    ops::{Range, RangeBounds},
    sync::{LazyLock, LockResult, Mutex, MutexGuard},
};
use thiserror::Error;

pub static USED_PORTS: LazyLock<UsedPorts> = LazyLock::new(UsedPorts::default);
pub(crate) const RANGE_PORTS: Range<u16> = 1000..(9999 + 1);

#[derive(Debug)]
pub struct Port<'a> {
    port: u16,
    source: &'a UsedPorts,
}

impl<'a> Port<'a> {
    pub const fn new(port: u16, source: &'a UsedPorts) -> Self {
        Self { port, source }
    }

    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for Port<'_> {
    fn drop(&mut self) {
        self.source
            .remove(self.port)
            .expect("Remove port is not valid");
    }
}

impl PartialEq for Port<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.port == other.port
    }
}

impl Eq for Port<'_> {}

impl PartialEq<u16> for Port<'_> {
    fn eq(&self, port: &u16) -> bool {
        self.port == *port
    }
}

impl PartialEq<u16> for &Port<'_> {
    fn eq(&self, port: &u16) -> bool {
        self.port == *port
    }
}

impl PartialEq<Port<'_>> for u16 {
    fn eq(&self, other: &Port<'_>) -> bool {
        *self == other.port()
    }
}

impl PartialEq<&Port<'_>> for u16 {
    fn eq(&self, other: &&Port<'_>) -> bool {
        *self == other.port()
    }
}

impl PartialOrd for Port<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.port.cmp(&other.port))
    }
}

impl PartialOrd<u16> for Port<'_> {
    fn partial_cmp(&self, other_port: &u16) -> Option<std::cmp::Ordering> {
        Some(self.port.cmp(other_port))
    }
}

impl PartialOrd<Port<'_>> for u16 {
    fn partial_cmp(&self, port: &Port<'_>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&port.port))
    }
}

impl std::fmt::Display for Port<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.port)
    }
}

impl<'a> From<Port<'a>> for u16 {
    fn from(port: Port<'a>) -> Self {
        port.port
    }
}

#[derive(Debug, Error)]
pub enum UsedPortsError {
    #[error("Mutex is poison")]
    Poison,

    #[error("Port not found")]
    PortNotFound,
}

pub(crate) type UsedPortsResult<T> = Result<T, UsedPortsError>;

#[derive(Debug, Default)]
pub struct UsedPorts {
    ports: Mutex<HashSet<u16>>,
}

impl<'a> UsedPorts {
    pub fn free(&'a self) -> UsedPortsResult<Port<'a>> {
        self.free_by_range(RANGE_PORTS)
    }

    pub fn free_by_range<R>(&'a self, range: R) -> UsedPortsResult<Port<'a>>
    where
        R: RangeBounds<u16> + IntoIterator<Item = u16>,
    {
        log::debug!(
            "Get free port by range: {:?}..{:?}",
            range.start_bound(),
            range.end_bound()
        );

        let mut ports = self.ports.lock().map_err(|_| UsedPortsError::Poison)?;

        for port in range {
            if ports.contains(&port) || !openport::is_free(port) {
                continue;
            }

            ports.insert(port);
            return Ok(Port::new(port, self));
        }

        Err(UsedPortsError::PortNotFound)
    }

    pub fn used(&self) -> UsedPortsResult<usize> {
        Ok(self.ports.lock().map_err(|_| UsedPortsError::Poison)?.len())
    }

    pub fn remove(&self, port: u16) -> UsedPortsResult<()> {
        log::debug!("Delete port: {port}");

        self.ports
            .lock()
            .map_err(|_| UsedPortsError::Poison)?
            .remove(&port);
        Ok(())
    }

    pub fn insert(&self, port: u16) -> UsedPortsResult<bool> {
        Ok(self
            .ports
            .lock()
            .map_err(|_| UsedPortsError::Poison)?
            .insert(port))
    }

    pub fn lock(&'a self) -> LockResult<MutexGuard<'a, HashSet<u16>>> {
        self.ports.lock()
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
    fn partial_eq_for_u16() {
        let used_port = UsedPorts::default();

        let port1 = used_port.free_by_range(1000..3000).unwrap();
        let port2 = 1;

        assert_ne!(port1, port2);
        assert_ne!(port2, port1);
        assert_eq!(port1, port1.port());
        assert_eq!(port1.port(), port1);

        let port1_ref = &port1;
        assert_ne!(port1_ref, port2);
        assert_ne!(port2, port1_ref);
        assert_eq!(port1_ref, port1_ref.port());
        assert_eq!(port1_ref.port(), port1_ref);
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
    fn partial_ord_for_u16() {
        let used_port = UsedPorts::default();

        let port1 = Port::new(1000, &used_port);
        let port2 = 1500;

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
    fn free_by_range() {
        let used_port = UsedPorts::default();
        let port = used_port.free_by_range(9900..=9999).unwrap();

        assert!(port >= 9900);
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

        assert_eq!(port, 5462);
    }

    #[test_log::test]
    fn insert() {
        let used_ports = UsedPorts::default();

        used_ports.insert(1000).unwrap();
        assert_eq!(used_ports.used().unwrap(), 1);
    }
}
