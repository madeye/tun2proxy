//! Session metadata describing a single TCP or UDP flow extracted by the
//! netstack.

use std::net::SocketAddr;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Protocol {
    Tcp,
    Udp,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub protocol: Protocol,
    pub src: SocketAddr,
    pub dst: SocketAddr,
    /// Best-effort hostname extracted from the first bytes of the flow
    /// (TLS SNI for TLS, `Host:` for HTTP/1). `None` for everything else.
    pub hostname_hint: Option<String>,
}

impl Session {
    pub fn new_tcp(src: SocketAddr, dst: SocketAddr) -> Self {
        Self {
            protocol: Protocol::Tcp,
            src,
            dst,
            hostname_hint: None,
        }
    }

    pub fn new_udp(src: SocketAddr, dst: SocketAddr) -> Self {
        Self {
            protocol: Protocol::Udp,
            src,
            dst,
            hostname_hint: None,
        }
    }
}
