use gielinor_protocol::{
    login::{self, Authentication},
    rsa::PrivateKey,
};
fn hex(s: &str) -> Vec<u8> {
    let s = s.trim();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn decodes_login_accepted_by_rsprot() {
    let payload = hex(include_str!(
        "../../../reference/protocol/login-fixture/login.hex"
    ));
    let key = PrivateKey::from_be_bytes(
        &hex(include_str!(
            "../../../reference/protocol/login-fixture/rsa-exponent.hex"
        )),
        &hex(include_str!(
            "../../../reference/protocol/login-fixture/rsa-modulus.hex"
        )),
    )
    .unwrap();
    let login = login::decode(&payload, &key).unwrap();
    let summary = format!(
        "{}\n{} {}\n{} {}\n{} {}\n{}\n{}\n{}\n",
        login.username,
        login.header.revision,
        login.header.subrevision,
        login.width,
        login.height,
        login.low_detail,
        login.resized,
        login.server_seed,
        login.affiliate_id,
        login.client_checksum
    );
    assert_eq!(
        summary,
        include_str!("../../../reference/protocol/login-fixture/login.expected")
    );
    assert_eq!(login.seed, [0x80000000, 239, 1, 0xffffffff]);
    for (i, crc) in login.cache_crcs.iter().enumerate() {
        assert_eq!(*crc, 0x80102030 + i as u32 * 0x010203);
    }
    assert_eq!(login.archive_checksums, [123, 456]);
    match login.authentication {
        Authentication::Password(p) => assert_eq!(p.as_bytes(), b"fixture-only"),
        _ => panic!("wrong authentication kind"),
    }
    // Truncated input must never panic or leak a partly accepted login.
    for end in [0, 1, 14, 15, 16, 20, payload.len() - 1] {
        assert!(
            login::decode(&payload[..end], &key).is_err(),
            "prefix {end}"
        );
    }
}
