//! Bidirectional packet pump between a TUN device and the user-space stack.
//!
//! Both pumps are written generically over `Stream<Item = io::Result<Vec<u8>>>`
//! and `Sink<Vec<u8>, Error = io::Error>` so unit tests can substitute
//! in-memory duplex pairs.

use std::io;

use futures::{Sink, SinkExt, Stream, StreamExt};

/// Forward every packet from `src` to `dst`. Returns `Ok(())` on clean EOF
/// of `src` or first error.
pub async fn pump<R, W>(mut src: R, mut dst: W) -> io::Result<()>
where
    R: Stream<Item = io::Result<Vec<u8>>> + Unpin,
    W: Sink<Vec<u8>, Error = io::Error> + Unpin,
{
    while let Some(pkt) = src.next().await {
        dst.send(pkt?).await?;
    }
    Ok(())
}
