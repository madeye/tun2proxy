//! Thin wrapper around [`netstack_smoltcp::StackBuilder`] with sane defaults.

use netstack_smoltcp::{Runner, StackBuilder, TcpListener, UdpSocket};

pub use netstack_smoltcp::{AnyIpPktFrame, Stack, TcpStream};

/// Tunables for the user-space TCP/IP stack.
#[derive(Debug, Clone)]
pub struct StackOptions {
    pub stack_buffer_size: usize,
    pub tcp_buffer_size: usize,
    pub enable_tcp: bool,
    pub enable_udp: bool,
    pub enable_icmp: bool,
}

impl Default for StackOptions {
    fn default() -> Self {
        Self {
            stack_buffer_size: 512,
            tcp_buffer_size: 4096,
            enable_tcp: true,
            enable_udp: true,
            enable_icmp: false,
        }
    }
}

/// All handles produced by [`build`]. The caller is expected to
/// `tokio::spawn(runner)` if it is `Some`, then drive the stack via
/// [`crate::relay`] and dispatch flows from `tcp_listener` / `udp_socket`.
pub struct StackHandles {
    pub stack: Stack,
    pub runner: Option<Runner>,
    pub tcp_listener: Option<TcpListener>,
    pub udp_socket: Option<UdpSocket>,
}

/// Build a netstack with the supplied options.
pub fn build(opts: &StackOptions) -> std::io::Result<StackHandles> {
    let (stack, runner, udp_socket, tcp_listener) = StackBuilder::default()
        .stack_buffer_size(opts.stack_buffer_size)
        .tcp_buffer_size(opts.tcp_buffer_size)
        .enable_tcp(opts.enable_tcp)
        .enable_udp(opts.enable_udp)
        .enable_icmp(opts.enable_icmp)
        .build()?;
    Ok(StackHandles {
        stack,
        runner,
        tcp_listener,
        udp_socket,
    })
}
