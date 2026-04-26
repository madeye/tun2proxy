# `tun2proxy` — Product Requirements Document

**Status:** Draft (M0)
**Owner:** Max Lv (`max.c.lv@gmail.com`)
**Last updated:** 2026-04-26

## 1. Problem statement

Many client environments (mobile VPN apps, network namespaces, sandboxes,
research lab boxes) need to transparently funnel **all** TCP and UDP
traffic of a process tree through a single upstream proxy without
configuring every individual application. The standard answer is a TUN
device whose packets are terminated by a user-space TCP/IP stack and
re-dialed through the proxy ("tun2socks").

Existing options each fall short:

- **`go-tun2socks`/`tun2socks-bin`** — Go runtime, awkward to embed in
  iOS/Android apps that already ship a Rust core.
- **`blechschmidt/tun2proxy`** — Rust, mature, but couples to `ipstack`
  and lacks HTTP/2 extended `CONNECT`. Not an embeddable library by
  design.
- **`mihomo-android`'s tun2socks crate** — internal to mihomo, not a
  general-purpose library.

We want a **clean, embeddable Rust crate** built on
[`netstack-smoltcp`](https://crates.io/crates/netstack-smoltcp) plus a
matching CLI, supporting modern proxy transports (HTTP/1.1, HTTPS, HTTP/2,
SOCKS5) with optional Basic auth.

## 2. Goals

- **G1.** Library (`tun2proxy-core` + `tun2proxy-proxy`) usable from
  Rust, Android (JNI cdylib), and iOS (staticlib).
- **G2.** Standalone CLI (`tun2proxy`) for macOS and Linux desktops,
  driven by flags or an optional TOML config.
- **G3.** Upstream proxy support:
  - SOCKS5 with `NoAuth` and `username/password`.
  - HTTP/1.1 `CONNECT` with optional Basic auth (`http://`).
  - HTTPS `CONNECT` (HTTP/1.1 over TLS) with optional Basic auth
    (`https://`).
  - HTTP/2 extended `CONNECT` (RFC 8441) with ALPN `h2` and optional
    Basic auth (`https+h2://`).
- **G4.** Transparent UDP forwarding (SOCKS5 UDP associate; UDP-over-TCP
  fallback for HTTP-only proxies).
- **G5.** Pre-push gate (`fmt + clippy + test`) green on macOS and
  Linux CI before any merge.
- **G6.** MIT license, single-author copyright, contribution-friendly.

## 3. Non-goals (v1)

- Windows / Wintun support.
- Rule-based routing (domain/CIDR rules, GeoIP). Use a single global
  proxy for v1.
- DoH / fake-IP / DNS hijacking. DNS is forwarded transparently through
  the proxy.
- Shadowsocks, VMess, Trojan, Hysteria, etc. Out of scope; users who
  need these wrap an upstream client like `mihomo` and point us at its
  local SOCKS5 listener.
- IPv6 e2e test rigs (the netstack supports v6 from day one; we just
  don't gate v1 on v6 conformance).

## 4. Users & use cases

| User | Use case |
|------|----------|
| Mobile app developer (Android/iOS) | Embed the library in a VPN service, hand it the TUN fd from the OS, point at a local mihomo SOCKS5. |
| Linux power user | Run `tun2proxy --proxy socks5://… --tun-name tun0` to funnel a netns through a remote proxy. |
| macOS power user | Same, with `utun`-backed device, no root required for read access (see `tun` crate docs). |
| Researcher | Reproducible packet capture against arbitrary proxies. |

## 5. Requirements

### 5.1 Functional

- **F1.** Accept a TUN device by name (`--tun-name`) **or** by
  pre-opened file descriptor (`--tun-fd`, used by FFI consumers).
- **F2.** Parse a single `--proxy <url>` URL and select the matching
  connector. URL schemes: `socks5`, `http`, `https`, `https+h2`.
  Username/password embedded in the URL are URL-decoded and forwarded
  to the connector.
- **F3.** TCP relay: every accepted netstack TCP flow is dialed through
  the connector and bidirectionally piped (`tokio::io::copy_bidirectional`).
- **F4.** UDP relay: SOCKS5 UDP associate; for HTTP-only proxies, fall
  back to per-flow UDP-over-TCP encapsulation (length-prefixed frames
  inside a `CONNECT` tunnel).
- **F5.** DNS: UDP/53 flows go through the same path as any other UDP
  flow. No special handling.
- **F6.** Graceful shutdown on SIGINT/SIGTERM (CLI) and on a `stop()`
  call (FFI handle).
- **F7.** Structured logging via `tracing`; level controlled by
  `RUST_LOG` and/or `--log-level`.

### 5.2 Non-functional

- **N1.** Throughput: ≥ 200 Mb/s single-stream TCP through a local
  SOCKS5 proxy on a 2024-era M-series Mac. (Soft target; revisit after
  M2.)
- **N2.** No `unsafe` outside `tun2proxy-ffi`.
- **N3.** MSRV: stable Rust 1.85.
- **N4.** All public API items documented with at least one rustdoc
  example by the time `v1.0.0` ships.

### 5.3 Security

- TLS server certificates verified against the OS trust store via
  `rustls-native-certs`. No `--insecure` flag in v1.
- Proxy passwords accepted via the URL **or** via env vars
  (`TUN2PROXY_PROXY_PASSWORD`); never logged at any level.
- Basic-auth headers redacted from `tracing` output.

## 6. URL grammar

```
proxy-url   = scheme "://" [ userinfo "@" ] host [ ":" port ]
scheme      = "socks5" | "http" | "https" | "https+h2"
userinfo    = pct-encoded-user [ ":" pct-encoded-password ]
```

Examples:

- `socks5://127.0.0.1:1080`
- `socks5://alice:s3cret@10.0.0.1:1080`
- `http://proxy.corp:3128`
- `https://alice:s3cret@proxy.corp:443`
- `https+h2://alice:s3cret@proxy.corp:443`

## 7. CLI surface (target, lands at M2/M6)

```
tun2proxy --proxy <url>
          (--tun-name <name> | --tun-fd <fd>)
          [--mtu <n>]                 # default 1500
          [--bypass <cidr>]...         # repeatable
          [--config <path.toml>]
          [--log-level <level>]
```

`--config` reads a TOML file whose keys mirror flag names. Flags win
over file values.

## 8. Architecture summary

```
+----------+   raw IP packets   +---------------------+
|   TUN    | <----------------> |  netstack-smoltcp   |
+----------+                    +----+-----------+----+
                                     |           |
                          TCP streams|           |UDP datagrams
                                     v           v
                              +-------------------------+
                              | session router (core)   |
                              +-----------+-------------+
                                          |
                                          v
                              +-------------------------+
                              | ProxyConnector (proxy)  |
                              +---+--------+--------+---+
                                  |        |        |
                              SOCKS5    HTTP     HTTP/2
                                       CONNECT   CONNECT
```

The session router is a single-owner task (mihomo pattern) reading
ingress and writing egress through mpsc channels — no `BiLock`, no
shared state on the hot path.

## 9. Open questions (tracked in `docs/TODO.md`)

- IPv6 default-on or feature-gated for M1?
- Should `--bypass` accept hostnames (which would re-introduce DNS rule
  matching) or remain CIDR-only?
- Reuse of HTTP/2 connections across many sessions — pool size knobs?
