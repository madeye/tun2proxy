# `tun2proxy` — TODO

Per-milestone task lists. Strike off items in the same PR that lands
the milestone. Open questions live at the bottom.

## M0 — `chore/bootstrap-docs`

- [x] Workspace `Cargo.toml` with shared `[workspace.dependencies]`.
- [x] `crates/tun2proxy-core` stub (`#![forbid(unsafe_code)]`, README).
- [x] `crates/tun2proxy-proxy` stub.
- [x] `crates/tun2proxy-cli` stub binary that initializes `tracing`.
- [x] `LICENSE` (MIT, Copyright (c) 2026 Max Lv).
- [x] Top-level `README.md`.
- [x] `CLAUDE.md` (guardrails) + `AGENTS.md` (mirror).
- [x] `docs/PRD.md`, `docs/ROADMAP.md`, `docs/TODO.md`.
- [x] `.gitignore`, `rustfmt.toml`, `clippy.toml`.
- [ ] `.github/workflows/ci.yml` (fmt + clippy + test on macOS + Ubuntu).
- [ ] Pre-push gate green: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all`.

## M1 — `feat/core-tun-netstack`

- [ ] `TunDevice` trait (`AsyncRead + AsyncWrite + Send`).
- [ ] Default impl using `tun = "0.8"` with `features = ["async"]`.
- [ ] `Stack` builder wrapping `netstack_smoltcp::StackBuilder`.
- [ ] Single-owner packet loop: TUN reader → stack ingress mpsc; stack
      egress mpsc → TUN writer; both joined under one `tokio::select!`.
- [ ] `Session { src, dst, protocol, hostname_hint }` type.
- [ ] Best-effort SNI / HTTP `Host` sniffer for `hostname_hint`.
- [ ] Loopback integration test: open a pair of in-memory TUN halves,
      push N packets, assert N received with no drops.
- [ ] CI: enable on macOS + Ubuntu.

## M2 — `feat/proxy-socks5`

- [ ] `ProxyConnector` trait + `Target` type.
- [ ] `ProxyUrl` parser (shared with M3/M4 connectors).
- [ ] `Socks5Connector` on `tokio-socks` (NoAuth + UserPass).
- [ ] CLI: `--proxy socks5://…`, `--tun-name`, `--tun-fd`.
- [ ] E2E test rig: `docker run --rm dperson/3proxy …` plus `curl`
      through the TUN.
- [ ] README quickstart shows a SOCKS5 example.

## M3 — `feat/proxy-http-connect`

- [ ] `HttpConnectConnector` (`httparse`, optional Basic auth).
- [ ] `HttpsConnectConnector` (TLS via `tokio-rustls` +
      `rustls-native-certs`).
- [ ] Header redaction for `Proxy-Authorization` in tracing output.
- [ ] E2E test against `mitmproxy` (HTTP) and `squid` (HTTP + HTTPS).

## M4 — `feat/proxy-http2-connect`

- [ ] `Http2ConnectConnector` using the `h2` crate, ALPN `h2`,
      `:protocol = "connect-tcp"`-style extended CONNECT.
- [ ] Per-target connection pool with idle eviction.
- [ ] E2E test against an HTTP/2 CONNECT proxy.

## M5 — `feat/udp-and-dns-forwarding`

- [ ] SOCKS5 UDP associate path.
- [ ] UDP-over-TCP fallback for HTTP-only proxies (length-prefixed
      datagrams inside a CONNECT tunnel).
- [ ] Transparent UDP/53 forwarding.
- [ ] E2E DNS resolution test through every connector.

## M6 — `feat/cli-config-and-toml`

- [ ] `--config <toml>` loader, schema mirrors flags.
- [ ] `--bypass <cidr>` (repeatable) for direct routing.
- [ ] `--log-level`, `--mtu`.
- [ ] `examples/example.toml` checked in.

## M7 — `feat/ffi-android-ios`

- [ ] New `crates/tun2proxy-ffi` (cdylib + staticlib).
- [ ] C header generation via `cbindgen` (build script).
- [ ] `tun2proxy_start(fd, proxy_url, ...) -> handle` /
      `tun2proxy_stop(handle)`.
- [ ] CI build matrix for `aarch64-linux-android`,
      `aarch64-apple-ios`, `aarch64-apple-ios-sim`,
      `aarch64-apple-darwin`.
- [ ] FFI README with sample Kotlin + Swift call sites.

## M8 — `feat/observability`

- [ ] `tracing` spans wrapping each TCP/UDP session.
- [ ] Optional Prometheus exporter on `--stats-addr`.
- [ ] Structured shutdown: SIGINT/SIGTERM, JoinSet draining, deadline.

## v1.0.0 — `release/v1.0.0`

- [ ] Bump `[workspace.package].version = "1.0.0"`.
- [ ] `CHANGELOG.md` from milestone history.
- [ ] `cargo publish --dry-run` per crate.
- [ ] Tag `v1.0.0`, push.

## Open questions (un-blocked decisions)

- IPv6 default-on or feature-gated at M1?
- Should `--bypass` accept hostnames (re-introduces DNS rule matching)
  or stay CIDR-only?
- HTTP/2 connection pool: per-target only, or also per-(target, auth)
  to handle credential rotation cleanly?
- FFI: callback-based logging into the host app, or always go through
  Rust `tracing` and let the host install a subscriber?
