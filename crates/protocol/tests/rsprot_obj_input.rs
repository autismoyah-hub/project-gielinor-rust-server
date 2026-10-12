use gielinor_protocol::obj_input;
#[test]
fn rsprot_all_ground_object_operations_and_suboperations_match() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/obj-input.expected"
    ))
    .expect("Run scripts/obj-input-parity.ps1");
    assert_eq!(fixture.lines().count(), 10240);
    for (case, line) in fixture.lines().enumerate() {
        let f = line.split('|').collect::<Vec<_>>();
        let opcode = f[0].parse().unwrap();
        let bytes = (0..f[1].len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&f[1][i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let v = obj_input::decode(opcode, &bytes).unwrap();
        assert_eq!(
            format!(
                "{},{},{},{},{},{}",
                v.id,
                v.x,
                v.z,
                v.operation,
                v.suboperation,
                u8::from(v.control)
            ),
            f[2],
            "case {case}"
        );
        assert!(obj_input::decode(opcode, &bytes[..7]).is_err());
        let mut extra = bytes;
        extra.push(0);
        assert!(obj_input::decode(opcode, &extra).is_err());
    }
}
