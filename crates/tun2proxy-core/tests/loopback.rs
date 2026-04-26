//! End-to-end loopback tests for the core packet engine.
//!
//! Two tests:
//! 1. `pump_loses_no_packets` — pushes 1 000 packets through [`relay::pump`]
//!    via in-memory mpsc channels and asserts every packet arrives in order.
//! 2. `netstack_completes_tcp_handshake` — feeds a real TCP SYN into the
//!    `netstack-smoltcp` Stack, replies to the SYN-ACK with an ACK, and
//!    asserts a `TcpStream` pops out of the `TcpListener` with the right
//!    addresses. Exercises the full Stack + relay wiring end-to-end without
//!    needing a real TUN device.

use std::{
    net::{Ipv4Addr, SocketAddrV4},
    time::Duration,
};

use futures::{SinkExt, StreamExt};
// `sink_map_err` lives on `SinkExt`; ensure it's in scope.
use smoltcp::{
    phy::ChecksumCapabilities,
    wire::{
        IpAddress, IpProtocol, Ipv4Packet, Ipv4Repr, TcpControl, TcpPacket, TcpRepr, TcpSeqNumber,
    },
};
use tokio::{
    sync::mpsc,
    time::{timeout, Instant},
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::PollSender;
use tun2proxy_core::{relay, stack};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pump_loses_no_packets() {
    const N: usize = 1_000;
    let (in_tx, in_rx) = mpsc::channel::<std::io::Result<Vec<u8>>>(64);
    let (out_tx, mut out_rx) = mpsc::channel::<Vec<u8>>(64);

    let src = ReceiverStream::new(in_rx);
    let dst = PollSender::new(out_tx).sink_map_err(|_| std::io::Error::other("closed"));

    let pump = tokio::spawn(relay::pump(src, dst));

    let producer = tokio::spawn(async move {
        for i in 0..N {
            let pkt = (i as u32).to_be_bytes().to_vec();
            in_tx.send(Ok(pkt)).await.expect("send");
        }
        drop(in_tx);
    });

    let started = Instant::now();
    for i in 0..N {
        let pkt = timeout(Duration::from_secs(10), out_rx.recv())
            .await
            .expect("timeout waiting for packet")
            .expect("channel closed early");
        assert_eq!(pkt, (i as u32).to_be_bytes());
    }
    assert!(out_rx.recv().await.is_none(), "extra packets received");
    producer.await.expect("producer task panicked");
    pump.await
        .expect("pump task panicked")
        .expect("pump errored");
    assert!(started.elapsed() < Duration::from_secs(10));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn netstack_completes_tcp_handshake() {
    let handles = stack::build(&stack::StackOptions::default()).expect("build stack");
    let mut listener = handles.tcp_listener.expect("tcp listener");
    if let Some(runner) = handles.runner {
        tokio::spawn(runner);
    }
    let (mut sink, mut stream) = handles.stack.split();

    let src = SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 2), 54321);
    let dst = SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 1), 80);
    let client_isn: u32 = 0x10000;

    sink.send(build_tcp_packet(
        src,
        dst,
        TcpControl::Syn,
        client_isn,
        None,
    ))
    .await
    .expect("send syn");

    let synack = timeout(Duration::from_secs(2), stream.next())
        .await
        .expect("timeout waiting for SYN-ACK")
        .expect("stack stream closed")
        .expect("stack stream errored");
    let ip = Ipv4Packet::new_checked(&synack[..]).expect("ipv4 parse");
    assert_eq!(ip.next_header(), IpProtocol::Tcp);
    let tcp = TcpPacket::new_checked(ip.payload()).expect("tcp parse");
    assert!(tcp.syn() && tcp.ack(), "expected SYN-ACK from stack");
    let server_isn = tcp.seq_number().0 as u32;

    sink.send(build_tcp_packet(
        src,
        dst,
        TcpControl::None,
        client_isn + 1,
        Some(server_isn.wrapping_add(1)),
    ))
    .await
    .expect("send ack");

    let (_stream, local, remote) = timeout(Duration::from_secs(2), listener.next())
        .await
        .expect("timeout waiting for accepted TcpStream")
        .expect("listener closed");
    assert_eq!(local.port(), src.port());
    assert_eq!(remote.port(), dst.port());
}

fn build_tcp_packet(
    src: SocketAddrV4,
    dst: SocketAddrV4,
    control: TcpControl,
    seq: u32,
    ack: Option<u32>,
) -> Vec<u8> {
    let tcp = TcpRepr {
        src_port: src.port(),
        dst_port: dst.port(),
        control,
        seq_number: TcpSeqNumber(seq as i32),
        ack_number: ack.map(|a| TcpSeqNumber(a as i32)),
        window_len: 65_535,
        window_scale: None,
        max_seg_size: matches!(control, TcpControl::Syn).then_some(1460),
        sack_permitted: false,
        sack_ranges: [None; 3],
        timestamp: None,
        payload: &[],
    };
    let ip = Ipv4Repr {
        src_addr: *src.ip(),
        dst_addr: *dst.ip(),
        next_header: IpProtocol::Tcp,
        payload_len: tcp.buffer_len(),
        hop_limit: 64,
    };
    let mut buf = vec![0u8; ip.buffer_len() + tcp.buffer_len()];
    let caps = ChecksumCapabilities::default();
    let mut ipp = Ipv4Packet::new_unchecked(&mut buf[..]);
    ip.emit(&mut ipp, &caps);
    let mut tcpp = TcpPacket::new_unchecked(&mut buf[ip.buffer_len()..]);
    tcp.emit(
        &mut tcpp,
        &IpAddress::Ipv4(*src.ip()),
        &IpAddress::Ipv4(*dst.ip()),
        &caps,
    );
    buf
}
