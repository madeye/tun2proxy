# `tun2proxy` — Roadmap

Each row is a single PR off `main`. The pre-push gate
(`fmt + clippy -D warnings + test`) must be green before merge.

When a milestone lands, move its row to **§ Done** and append the merge
commit hash.

## Planned

| #   | Branch                          | Scope                                                                                                                                            | Exit criteria                                                              |
| --- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------- |
| M2  | `feat/proxy-socks5`             | `Socks5Connector` (NoAuth + UserPass) on `tokio-socks`. CLI `--proxy socks5://…`. e2e test against a `3proxy` Docker container.                  | TCP relay e2e through SOCKS5 with and without auth; clippy clean.          |
| M3  | `feat/proxy-http-connect`       | `HttpConnectConnector` (HTTP/1.1) and `HttpsConnectConnector` (TLS-wrapped). Optional Basic auth. ProxyUrl parser shared across connectors.      | E2E test through `mitmproxy` and `squid` (HTTP and HTTPS).                 |
| M4  | `feat/proxy-http2-connect`      | `Http2ConnectConnector` (RFC 8441 extended `CONNECT`) with ALPN `h2` and connection multiplexing. Per-target connection pool.                    | E2E test against an `nginx`/`envoy` HTTP/2 `CONNECT` proxy.                |
| M5  | `feat/udp-and-dns-forwarding`   | UDP relay path: SOCKS5 UDP associate; per-flow UDP-over-TCP fallback for HTTP-only proxies. Transparent DNS (UDP/53) forwarding.                  | DNS resolution works through every connector type in the e2e rig.          |
| M6  | `feat/cli-config-and-toml`      | `--config <toml>` loader, `--bypass <cidr>`, `--log-level`, `--mtu`. TOML schema mirrors flags; flags win.                                       | `tun2proxy --config example.toml` starts a session.                        |
| M7  | `feat/ffi-android-ios`          | New `tun2proxy-ffi` crate (cdylib + staticlib). C header via `cbindgen`. fd-based start/stop. Build matrix: `aarch64-linux-android`, `aarch64-apple-ios`, `aarch64-apple-ios-sim`, `aarch64-apple-darwin`. | All targets build green; smoke load from a stub Swift/Kotlin app deferred to user. |
| M8  | `feat/observability`            | `tracing` spans per session; optional Prometheus stats endpoint (`--stats-addr`); structured shutdown signal handling.                           | `curl http://127.0.0.1:9090/metrics` returns expected counters.            |
| v1  | `release/v1.0.0`                | Version bump to `1.0.0`, `CHANGELOG.md`, publish all crates to crates.io.                                                                        | `cargo publish --dry-run` clean for every crate; tag pushed.               |

## Done

| #   | Branch                          | Scope                                                                                                                                            | Exit criteria                                                              | Merge      |
| --- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------- | ---------- |
| M0  | `chore/bootstrap-docs`          | Workspace skeleton + crate stubs, `LICENSE`, `README.md`, `CLAUDE.md`, `AGENTS.md`, `docs/{PRD,ROADMAP,TODO}.md`, `.github/workflows/ci.yml`, `.gitignore`, `rustfmt.toml`, `clippy.toml`. | `cargo check --workspace` succeeds; pre-push gate green; docs render.       | PR #1      |
| M1  | `feat/core-tun-netstack`        | TUN device wrapper, `netstack-smoltcp` wiring, packet-pump loop, `Session` + SNI/HTTP-Host sniffer, integration test that completes a TCP 3-way handshake against the netstack. | 1 000-packet pump test + TCP 3WHS test green on macOS + Linux.             | PR #2 _(filled at merge)_ |

## Deferred to v2+

- Windows / Wintun support.
- Rule-based routing (domain/CIDR matchers, GeoIP).
- DoH resolver and fake-IP DNS.
- Shadowsocks / VMess / Trojan / Hysteria connectors (or document
  pairing with `mihomo` as the canonical answer).
- WebTransport / QUIC `CONNECT` (HTTP/3).
