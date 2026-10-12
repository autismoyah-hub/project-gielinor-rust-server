use gielinor_protocol::js5::{self, Command};
fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn rsprot_js5_handshake_commands_and_continuation_blocks_match() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/js5.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 3072);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        match f[0] {
            "H" => {
                let bytes = hex(f[1]);
                let h = js5::handshake(&bytes).unwrap();
                assert_eq!(h.revision.to_string(), f[2]);
                assert_eq!(h.seed.map(|v| v.to_string()).join(","), f[3]);
                assert!(js5::handshake(&bytes[..19]).is_err());
            }
            "C" => {
                let op = f[1].parse().unwrap();
                let bytes = hex(f[2]);
                let result = js5::command(op, &bytes).unwrap();
                let value = match result {
                    Command::Request { archive, group, .. } => {
                        (((archive as u32) << 16) | group as u32).to_string()
                    }
                    Command::Priority { high } => if high { "high" } else { "low" }.into(),
                    Command::Xor(key) => key.to_string(),
                };
                assert_eq!(value, f[3], "command {case}");
                assert!(js5::command(op, &bytes[..2]).is_err());
            }
            "R" => {
                let mut actual =
                    js5::response(f[1].parse().unwrap(), f[2].parse().unwrap(), &hex(f[3]));
                assert_eq!(actual, hex(f[4]), "response {case}");
                let key = case as u8;
                js5::xor(&mut actual, key);
                js5::xor(&mut actual, key);
                assert_eq!(actual, hex(f[4]));
            }
            _ => panic!("unknown case"),
        }
    }
    assert!(js5::command(5, &[0; 3]).is_err());
}
