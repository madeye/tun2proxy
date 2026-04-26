#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

//! Upstream proxy connectors for `tun2proxy`.
//!
//! - `socks5` — SOCKS5 with no-auth and username/password (M2).
//! - `http1`  — HTTP/1.1 `CONNECT` with optional Basic auth (M3).
//! - `https`  — HTTP/1.1 `CONNECT` over TLS (M3).
//! - `http2`  — RFC 8441 extended `CONNECT` over HTTP/2 + ALPN (M4).
