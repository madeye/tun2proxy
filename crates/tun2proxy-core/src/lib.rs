#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

//! Core engine for `tun2proxy`. Wires a TUN device to `netstack-smoltcp`
//! and exposes the per-flow handles consumed by the upstream proxy
//! connectors in `tun2proxy-proxy`.

pub mod relay;
pub mod session;
pub mod sniff;
pub mod stack;
pub mod tun;

pub use session::{Protocol, Session};
pub use sniff::sniff;
pub use stack::{AnyIpPktFrame, Stack, StackHandles, StackOptions, TcpStream};
pub use tun::{open as open_tun, FramedTun, TunOptions, DEFAULT_MTU};
