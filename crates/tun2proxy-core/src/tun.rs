//! TUN device helpers.

use std::net::IpAddr;

use tokio_util::codec::Framed;
use tun::{AsyncDevice, Configuration, TunPacketCodec};

/// Default MTU used when the caller does not override it.
pub const DEFAULT_MTU: u16 = tun::DEFAULT_MTU;

/// Options for opening a TUN device by name.
#[derive(Debug, Clone)]
pub struct TunOptions {
    pub name: Option<String>,
    pub mtu: u16,
    pub address: Option<IpAddr>,
    pub netmask: Option<IpAddr>,
    pub destination: Option<IpAddr>,
}

impl Default for TunOptions {
    fn default() -> Self {
        Self {
            name: None,
            mtu: DEFAULT_MTU,
            address: None,
            netmask: None,
            destination: None,
        }
    }
}

/// Stream + Sink of raw IP packets.
pub type FramedTun = Framed<AsyncDevice, TunPacketCodec>;

/// Open a TUN device with the given options and return a [`Framed`] wrapper
/// that yields and accepts raw IP packets as `Vec<u8>`.
///
/// Requires elevated privileges on Linux (`CAP_NET_ADMIN`) and macOS.
pub fn open(opts: &TunOptions) -> std::io::Result<FramedTun> {
    let mut cfg = Configuration::default();
    cfg.mtu(opts.mtu).up();
    if let Some(name) = &opts.name {
        cfg.tun_name(name);
    }
    if let Some(addr) = opts.address {
        cfg.address(addr);
    }
    if let Some(mask) = opts.netmask {
        cfg.netmask(mask);
    }
    if let Some(dst) = opts.destination {
        cfg.destination(dst);
    }
    let device = tun::create_as_async(&cfg).map_err(std::io::Error::other)?;
    Ok(device.into_framed())
}
