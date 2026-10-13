use gielinor_protocol::{
    login,
    rsa::PrivateKey,
    session::{Event, Session, State},
};
fn hex(s: &str) -> Vec<u8> {
    let s = s.trim();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn fixture() -> (PrivateKey, Vec<u8>) {
    let root = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../reference/protocol/reconnect-fixture/"
    );
    let read = |name: &str| {
        std::fs::read_to_string(format!("{root}{name}")).expect("Run scripts/reconnect-parity.ps1")
    };
    (
        PrivateKey::from_be_bytes(
            &hex(&read("rsa-exponent.hex")),
            &hex(&read("rsa-modulus.hex")),
        )
        .unwrap(),
        hex(&read("login.hex")),
    )
}
#[test]
fn original_rsprot_reconnect_envelope_and_previous_session_key() {
    let (key, payload) = fixture();
    let block = login::decode_reconnect(&payload, &key).unwrap();
    let fixture_root = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../reference/protocol/reconnect-fixture/"
    );
    let expected = std::fs::read_to_string(format!("{fixture_root}login.expected")).unwrap();
    assert_eq!(
        format!(
            "{}\n{} {}\n{} {}\n{} {}\n{}\n{}\n{}\n",
            block.username,
            block.header.revision,
            block.header.subrevision,
            block.width,
            block.height,
            block.low_detail,
            block.resized,
            block.server_seed,
            block.affiliate_id,
            block.client_checksum
        ),
        expected
    );
    let previous = std::fs::read_to_string(format!("{fixture_root}previous-key.expected")).unwrap();
    assert_eq!(
        format!("{:?}", block.authentication.map(|v| v as i32)),
        previous
    );
    assert_eq!(block.username, "parity_actor");
    assert_eq!(block.seed, [0x80000000, 239, 1, u32::MAX]);
    assert_eq!(block.authentication, [u32::MAX, 0, 0x80000000, 2391]);
    assert_eq!(block.server_seed, 0x1020304050607080);
    assert_eq!((block.header.revision, block.header.subrevision), (239, 1));
    assert_eq!((block.width, block.height), (765, 503));
    assert!(block.low_detail && block.resized);
    assert_eq!(block.archive_checksums, [123, 456]);
    for (i, crc) in block.cache_crcs.iter().enumerate() {
        assert_eq!(*crc, 0x80102030 + i as u32 * 0x010203);
    }
    for end in [0, 1, 14, 15, 16, 20, payload.len() - 1] {
        assert!(
            login::decode_reconnect(&payload[..end], &key).is_err(),
            "prefix {end}"
        );
    }
}
fn status(key: &PrivateKey, payload: &[u8], seed: u64, chunk: usize) -> u8 {
    let mut wire = vec![14, 18];
    wire.extend((payload.len() as u16).to_be_bytes());
    wire.extend(payload);
    let mut session = Session::new(seed, [0; 23]);
    let mut rejected = None;
    for bytes in wire.chunks(chunk) {
        for event in session.receive(bytes, key).unwrap() {
            match event {
                Event::Rejected(code) => {
                    assert!(rejected.replace(code).is_none());
                }
                Event::Write(_) => {}
                _ => panic!("Reconnect must not admit a player"),
            }
        }
    }
    assert_eq!(session.state(), State::Closed);
    rejected.unwrap()
}
#[test]
fn reconnect_validates_before_reference_connect_fail_and_never_admits() {
    let (key, payload) = fixture();
    for chunk in [1, 2, 3, 8, 64, 4096] {
        assert_eq!(status(&key, &payload, 0x1020304050607080, chunk), 50);
        assert_eq!(status(&key, &payload, 1, chunk), 22);
    }
    assert_eq!(status(&key, &[], 1, 1), 67);
    let mut wrong_revision = payload.clone();
    wrong_revision[..4].copy_from_slice(&238u32.to_be_bytes());
    assert_eq!(status(&key, &wrong_revision, 1, 4096), 6);
    assert_eq!(status(&key, &payload[..20], 1, 4096), 67);
}
