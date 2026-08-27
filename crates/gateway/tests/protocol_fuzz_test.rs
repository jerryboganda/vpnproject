//! Protocol & Parser Fuzz / Robustness Test

use bytes::BytesMut;
use tokio_util::codec::Decoder;
use vpnbridge_gateway::socks5::parser::parse_udp_header;
use vpnbridge_protocol::codec::ProtocolCodec;

#[test]
fn test_fuzz_protocol_codec_random_inputs() {
    let mut codec = ProtocolCodec;

    // Test truncated inputs
    for len in 0..10 {
        let mut buf = BytesMut::from(&vec![0u8; len][..]);
        let _ = codec.decode(&mut buf);
    }

    // Test huge length prefix (should fail immediately on length check without allocating)
    let mut huge_len_buf = BytesMut::from(&[0xFF, 0xFF, 0xFF, 0xFF, 0x01, 0x02][..]);
    let res = codec.decode(&mut huge_len_buf);
    assert!(res.is_err(), "Huge frame lengths must be rejected");

    // Test garbage inputs
    let garbage: Vec<Vec<u8>> = vec![
        vec![0x00, 0x00, 0x00, 0x05, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        vec![0x00, 0x00, 0x00, 0x02, b'{', b'}'],
        vec![0x00, 0x00, 0x00, 0x04, b'n', b'u', b'l', b'l'],
    ];

    for g in garbage {
        let mut buf = BytesMut::from(&g[..]);
        let _ = codec.decode(&mut buf);
    }
}

#[test]
fn test_fuzz_udp_header_parser() {
    // Truncated UDP headers
    for len in 0..15 {
        let buf = vec![0u8; len];
        let _ = parse_udp_header(&buf);
    }

    // Invalid reserved bytes
    let invalid_rsv = [0x01, 0x00, 0x00, 0x01, 127, 0, 0, 1, 0x1F, 0x90];
    assert!(parse_udp_header(&invalid_rsv).is_err());

    // Fragmented UDP (unsupported)
    let frag = [0x00, 0x00, 0x01, 0x01, 127, 0, 0, 1, 0x1F, 0x90];
    assert!(parse_udp_header(&frag).is_err());
}
