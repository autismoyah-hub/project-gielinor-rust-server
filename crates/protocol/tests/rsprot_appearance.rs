use gielinor_protocol::appearance::{Appearance, Face, FaceTarget, Updates};
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
#[test]
fn original_rsprot_appearance_and_unified_facing() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/appearance.expected"
    ))
    .expect("Run scripts/appearance-parity.ps1");
    assert_eq!(fixture.lines().count(), 1024);
    let npc_faces = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/appearance.expected.npcface"
    ))
    .expect("Run scripts/appearance-parity.ps1");
    assert_eq!(npc_faces.lines().count(), 512);
    for (case, line) in fixture.lines().chain(npc_faces.lines()).enumerate() {
        let f: Vec<_> = line.split_whitespace().collect();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        if f[0] == "appearance" {
            let list = |i: usize| {
                f[i].split(',')
                    .map(|s| s.parse::<u16>().unwrap())
                    .collect::<Vec<_>>()
            };
            let value = Appearance {
                body_type: n(1) as u8,
                skull: n(2) as i8,
                overhead: n(3) as i8,
                combat: n(4) as u8,
                skill: n(5) as u16,
                pronoun: n(6) as u8,
                hidden: f[7] == "true",
                force_refresh: f[8] == "true",
                npc: (n(9) >= 0).then_some(n(9) as u16),
                name: format!("Parity_{}", n(10)),
                equipment: list(11).try_into().unwrap(),
                base_equipment: list(12).try_into().unwrap(),
                colours: list(13)
                    .into_iter()
                    .map(|n| n as u8)
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
                animations: list(14).try_into().unwrap(),
                name_extras: ["Pre".into(), "Post".into(), "Level".into()],
            };
            let raw = value.encode().unwrap();
            let block = Updates {
                appearance: Some(&raw),
                ..Default::default()
            }
            .encode()
            .unwrap();
            assert_eq!(hex(&block[1..]), f[15], "appearance {case}: {line}");
        } else {
            let target = match n(3) {
                0 => FaceTarget::Entity {
                    entity_type: n(4) as u16,
                    index: (n(5) >= 0).then_some(n(5) as u32),
                    fallback_angle: n(6) as u16,
                },
                1 => FaceTarget::Location {
                    x: n(7) as u16,
                    z: n(8) as u16,
                    width: n(9) as u8,
                    length: n(10) as u8,
                },
                2 => FaceTarget::Angle(n(6) as u16),
                3 => FaceTarget::Reset,
                _ => panic!(),
            };
            let face = Face {
                walk_mode: n(1) as u8,
                instant: f[2] == "true",
                target,
            };
            let bytes = if f[0] == "npcface" {
                face.encode_npc()
            } else {
                face.encode()
            }
            .unwrap();
            assert_eq!(hex(&bytes), f[11], "face {case}: {line}");
        }
    }
}
