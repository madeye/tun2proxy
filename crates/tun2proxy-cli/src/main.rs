#![forbid(unsafe_code)]

//! `tun2proxy` CLI. M0 ships a stub; full clap definition lands with M2.

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    tracing::info!("tun2proxy {} (M0 stub)", env!("CARGO_PKG_VERSION"));
    Ok(())
}
