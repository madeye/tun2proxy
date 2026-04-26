# tun2proxy

A Rust library and CLI that turns a TUN device into a transparent client
for upstream HTTP / HTTPS / HTTP-2 / SOCKS5 proxies. Built on top of
[`netstack-smoltcp`](https://crates.io/crates/netstack-smoltcp).

> **Status: M0 — bootstrap.** No functional code yet. Track progress in
> [`docs/ROADMAP.md`](docs/ROADMAP.md).

## Goals

- Embeddable Rust library (`tun2proxy-core` + `tun2proxy-proxy`) usable
  from desktop binaries and from Android (cdylib) / iOS (staticlib)
  apps that hand it a pre-opened TUN file descriptor.
- Standalone CLI (`tun2proxy`) for macOS and Linux.
- Modern proxy transports out of the box, all with optional Basic
  auth:
  - SOCKS5 (`socks5://`) — `NoAuth` and `username/password`.
  - HTTP/1.1 `CONNECT` (`http://`).
  - HTTPS `CONNECT` over TLS (`https://`).
  - HTTP/2 extended `CONNECT` per RFC 8441 (`https+h2://`).
- Transparent UDP forwarding (SOCKS5 UDP associate; UDP-over-TCP
  fallback for HTTP-only proxies).

See [`docs/PRD.md`](docs/PRD.md) for the full requirements.

## Non-goals (v1)

- Windows / Wintun (planned for v2).
- Rule-based routing, DoH, fake-IP DNS — pair with `mihomo` if you
  need those.
- Shadowsocks / VMess / Trojan / Hysteria — point us at the local
  SOCKS5 listener of an upstream client that speaks them.

## Workspace layout

```
crates/
├── tun2proxy-core   — TUN ↔ netstack-smoltcp ↔ session router
├── tun2proxy-proxy  — SOCKS5 / HTTP-1.1 / HTTPS / HTTP-2 connectors
└── tun2proxy-cli    — clap-based binary
```

`tun2proxy-ffi` (cdylib + staticlib for Android/iOS) lands in M7.

## Quickstart (target — lands at M2)

```sh
# macOS
sudo tun2proxy --proxy socks5://127.0.0.1:1080 --tun-name utun7

# Linux
sudo tun2proxy --proxy https://alice:s3cret@proxy.corp:443 --tun-name tun0
```

## Build

```sh
cargo build --workspace
```

Pre-push gate (run before every push — see
[`CLAUDE.md`](CLAUDE.md)):

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

## License

MIT — see [`LICENSE`](LICENSE).

## References

- [`netstack-smoltcp`](https://github.com/cavivie/netstack-smoltcp) —
  the user-space TCP/IP stack we sit on top of.
- [`mihomo-android`](https://github.com/MetaCubeX/mihomo) — internal
  Rust tun2socks crate; we borrow its single-owner Stack pattern.
- [`blechschmidt/tun2proxy`](https://github.com/blechschmidt/tun2proxy)
  — prior Rust art using `ipstack`.
