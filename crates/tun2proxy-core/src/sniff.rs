//! Best-effort hostname extraction from the first bytes of a TCP flow.
//!
//! Two protocols are recognized:
//! - TLS ClientHello → SNI extension (`server_name`, name_type 0).
//! - HTTP/1 plaintext → `Host:` header.
//!
//! Both functions are non-allocating apart from the returned `String`,
//! tolerate truncated input by returning `None`, and never panic on
//! malformed bytes.

/// Try every known sniffer in turn and return the first hit.
pub fn sniff(buf: &[u8]) -> Option<String> {
    sniff_tls_sni(buf).or_else(|| sniff_http_host(buf))
}

/// Extract the SNI host from a TLS ClientHello record.
pub fn sniff_tls_sni(buf: &[u8]) -> Option<String> {
    // TLS record header: type(1) + version(2) + length(2). type 0x16 = handshake.
    if buf.len() < 5 || buf[0] != 0x16 {
        return None;
    }
    let record_len = u16::from_be_bytes([buf[3], buf[4]]) as usize;
    let body = buf.get(5..5 + record_len)?;

    // Handshake header: type(1) + length(3). type 0x01 = ClientHello.
    if body.len() < 4 || body[0] != 0x01 {
        return None;
    }
    let hs_len = u32::from_be_bytes([0, body[1], body[2], body[3]]) as usize;
    let hs = body.get(4..4 + hs_len)?;

    // ClientHello body: version(2) + random(32) + session_id_len(1)+id +
    //                   cipher_suites_len(2)+cs + compression_len(1)+cm +
    //                   extensions_len(2) + extensions.
    let mut p = 0;
    let session_id_len = *hs.get(p + 34)? as usize;
    p = p + 35 + session_id_len;
    let cs_len = u16::from_be_bytes([*hs.get(p)?, *hs.get(p + 1)?]) as usize;
    p = p + 2 + cs_len;
    let cm_len = *hs.get(p)? as usize;
    p = p + 1 + cm_len;
    let ext_total = u16::from_be_bytes([*hs.get(p)?, *hs.get(p + 1)?]) as usize;
    p += 2;
    let ext_end = p.checked_add(ext_total)?;
    let exts = hs.get(p..ext_end)?;

    parse_sni_extensions(exts)
}

fn parse_sni_extensions(mut exts: &[u8]) -> Option<String> {
    while exts.len() >= 4 {
        let ty = u16::from_be_bytes([exts[0], exts[1]]);
        let len = u16::from_be_bytes([exts[2], exts[3]]) as usize;
        let body = exts.get(4..4 + len)?;
        if ty == 0x0000 {
            // server_name extension: list_len(2), list of (type(1), len(2), bytes).
            let list_len = u16::from_be_bytes([*body.first()?, *body.get(1)?]) as usize;
            let list = body.get(2..2 + list_len)?;
            let mut q = 0;
            while q + 3 <= list.len() {
                let name_type = list[q];
                let name_len = u16::from_be_bytes([list[q + 1], list[q + 2]]) as usize;
                let name = list.get(q + 3..q + 3 + name_len)?;
                if name_type == 0 {
                    return std::str::from_utf8(name).ok().map(str::to_owned);
                }
                q += 3 + name_len;
            }
            return None;
        }
        exts = &exts[4 + len..];
    }
    None
}

/// Extract the `Host:` header from an HTTP/1 request prefix.
pub fn sniff_http_host(buf: &[u8]) -> Option<String> {
    let head_end = buf.windows(4).position(|w| w == b"\r\n\r\n")?;
    let head = std::str::from_utf8(buf.get(..head_end)?).ok()?;
    // Skip the request line; it has no colon and a `?` short-circuit would
    // abort the scan before reaching `Host:`.
    for line in head.split("\r\n").skip(1) {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.trim().eq_ignore_ascii_case("Host") {
            return Some(value.trim().to_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_host_matches() {
        let req = b"GET / HTTP/1.1\r\nHost: example.com\r\nUA: x\r\n\r\n";
        assert_eq!(sniff_http_host(req).as_deref(), Some("example.com"));
        assert_eq!(sniff(req).as_deref(), Some("example.com"));
    }

    #[test]
    fn http_host_case_insensitive() {
        let req = b"GET / HTTP/1.1\r\nhost: example.org\r\n\r\n";
        assert_eq!(sniff_http_host(req).as_deref(), Some("example.org"));
    }

    #[test]
    fn http_truncated_returns_none() {
        let req = b"GET / HTTP/1.1\r\nHost: example.com\r\n";
        assert!(sniff_http_host(req).is_none());
    }

    #[test]
    fn tls_sni_matches() {
        // Minimal valid ClientHello with SNI = "example.com". Synthesized to
        // exercise the parser end-to-end without pulling in a TLS library.
        let host = b"example.com";
        let mut hello = Vec::new();
        hello.push(0x01); // handshake type ClientHello
        let body_start = hello.len();
        hello.extend_from_slice(&[0; 3]); // hs length placeholder
        hello.extend_from_slice(&[0x03, 0x03]); // legacy_version TLS 1.2
        hello.extend_from_slice(&[0u8; 32]); // random
        hello.push(0); // session_id length 0
        hello.extend_from_slice(&[0, 2, 0x13, 0x01]); // cipher suites: TLS_AES_128_GCM_SHA256
        hello.extend_from_slice(&[1, 0]); // compression methods: null
                                          // Build extensions: only server_name.
        let mut exts = Vec::new();
        exts.extend_from_slice(&[0x00, 0x00]); // ext_type = server_name
        let sn_body_len = (5 + host.len()) as u16;
        exts.extend_from_slice(&sn_body_len.to_be_bytes()); // ext_length
        exts.extend_from_slice(&((3 + host.len()) as u16).to_be_bytes()); // server_name_list_length
        exts.push(0); // name_type = host_name
        exts.extend_from_slice(&(host.len() as u16).to_be_bytes());
        exts.extend_from_slice(host);
        hello.extend_from_slice(&(exts.len() as u16).to_be_bytes());
        hello.extend_from_slice(&exts);
        // Patch handshake length (3 bytes, big-endian).
        let hs_len = (hello.len() - body_start - 3) as u32;
        hello[body_start] = ((hs_len >> 16) & 0xff) as u8;
        hello[body_start + 1] = ((hs_len >> 8) & 0xff) as u8;
        hello[body_start + 2] = (hs_len & 0xff) as u8;
        // Wrap in a TLS record.
        let mut record = Vec::new();
        record.push(0x16); // handshake
        record.extend_from_slice(&[0x03, 0x01]); // legacy_record_version
        record.extend_from_slice(&(hello.len() as u16).to_be_bytes());
        record.extend_from_slice(&hello);

        assert_eq!(sniff_tls_sni(&record).as_deref(), Some("example.com"));
        assert_eq!(sniff(&record).as_deref(), Some("example.com"));
    }

    #[test]
    fn tls_truncated_returns_none() {
        assert!(sniff_tls_sni(&[0x16, 0x03, 0x01, 0x00]).is_none());
        assert!(sniff_tls_sni(&[]).is_none());
        assert!(sniff_tls_sni(b"plain text").is_none());
    }
}
