#[test]
fn drop_dialog_packets_match_rsprot() {
    let rows = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/drop-dialog.expected"
    ))
    .expect("Run scripts/drop-dialog-parity.ps1");
    assert_eq!(rows.lines().count(), 4096);
    for line in rows.lines() {
        let p: Vec<_> = line.split('|').collect();
        if p[0] == "object" {
            let n: Vec<_> = p[1].split(',').collect();
            let packet = gielinor_protocol::interfaces::object(
                n[0].parse().unwrap(),
                n[1].parse().unwrap(),
                n[2].parse().unwrap(),
            );
            let actual: String = packet.payload.iter().map(|v| format!("{v:02x}")).collect();
            assert_eq!(actual, p[2], "{line}");
        } else {
            let bytes: Vec<u8> = p[1]
                .as_bytes()
                .chunks_exact(2)
                .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
                .collect();
            let pause = gielinor_protocol::dialog::pause(&bytes).unwrap();
            assert_eq!(
                format!("{},{}", pause.component, pause.subcomponent),
                p[2],
                "{line}"
            );
            assert!(gielinor_protocol::dialog::pause(&bytes[..5]).is_err());
        }
    }
}
