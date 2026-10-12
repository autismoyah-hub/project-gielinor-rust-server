use gielinor_protocol::zone::{self, Add, Object};
fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn actual_desktop_zone_enclosures_match_all_ground_object_fields() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/zone-wire.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 2048);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let v: Vec<i32> = f[0].split(',').map(|n| n.parse().unwrap()).collect();
        let (x, z, id) = (v[3] as u8, v[4] as u8, v[5] as u16);
        let objects = [
            Object::Add(Add {
                id,
                count: v[6],
                x,
                z,
                ops: v[9] as u8,
                until_public: v[10] as u16,
                until_despawn: v[11] as u16,
                ownership: v[12] as u8,
                never_public: v[13] != 0,
            }),
            Object::Delete {
                id,
                count: v[6],
                x,
                z,
            },
            Object::Count {
                id,
                old: v[7],
                new: v[8],
                x,
                z,
            },
        ];
        let reset = zone::reset(v[0] as u8, v[1] as u8, v[2] as u8);
        assert_eq!(reset.opcode, 54);
        assert_eq!(reset.payload, hex(f[1]), "reset {case}");
        let body = objects
            .iter()
            .flat_map(|o| o.encode().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(body, hex(f[2]), "body {case}");
        let enclosed = zone::enclosed(v[0] as u8, v[1] as u8, v[2] as u8, &objects).unwrap();
        assert_eq!(enclosed.opcode, 105);
        assert_eq!(enclosed.payload, hex(f[3]), "enclosure {case}");
    }
}
#[test]
fn enclosed_cache_limit_and_invalid_zone_coordinates_are_explicit() {
    let object = Object::Add(Add::visible(1, 1, 7, 7));
    assert!(zone::enclosed(0, 0, 0, &vec![object; 2666]).is_ok());
    assert!(zone::enclosed(0, 0, 0, &vec![object; 2667]).is_err());
    assert!(Object::Add(Add::visible(1, 1, 8, 0)).encode().is_err());
}
