# tun2proxy-core

Core engine for the `tun2proxy` workspace. Owns the TUN device, the
`netstack-smoltcp` stack, and the session-relay loop. Pluggable upstream
proxy clients are supplied by the `tun2proxy-proxy` crate.

Status: **stub** (M0). First functional code lands in M1.
