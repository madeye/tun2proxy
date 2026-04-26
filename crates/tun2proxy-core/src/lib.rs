#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

//! Core engine for `tun2proxy`. Wires a TUN device to `netstack-smoltcp`
//! and dispatches each accepted TCP/UDP flow to a `ProxyConnector`
//! supplied by the `tun2proxy-proxy` crate.
//!
//! The implementation lands in M1 (`feat/core-tun-netstack`).
