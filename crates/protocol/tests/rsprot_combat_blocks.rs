use gielinor_protocol::combat_updates::{self, Headbar, Hitmark};
#[test]
fn actual_rsprot_composite_masks_and_body_order() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/combat-updates.expected.composite"
    ))
    .expect("Run combat-updates-parity.ps1");
    assert_eq!(fixture.lines().count(), 1024);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let npc = f[0] == "N";
        let bits: u8 = f[1].parse().unwrap();
        let observer = f[2].parse().unwrap();
        let target = f[3].parse().unwrap();
        let hits = combat_updates::hitmarks(
            npc,
            observer,
            target,
            &[Hitmark {
                source_index: 65537,
                source_type: 18,
                self_type: 16,
                other_type: 17,
                value: 25,
                delay: 0,
                limit: 10,
            }],
        )
        .unwrap();
        let bars = combat_updates::headbars(
            npc,
            observer,
            target,
            &[Headbar {
                source_index: 65537,
                self_type: 1,
                other_type: 2,
                start_fill: 20,
                end_fill: 10,
                start_time: 0,
                end_time: 5,
            }],
        )
        .unwrap();
        let hitmarks = (bits & 1 != 0).then_some(hits.as_slice());
        let headbars = (bits & 2 != 0).then_some(bars.as_slice());
        let sequence = (bits & 4 != 0).then(|| (f[4].parse().unwrap(), f[5].parse().unwrap()));
        let face = (bits & 8 != 0).then_some(gielinor_protocol::appearance::Face {
            walk_mode: 0,
            instant: false,
            target: gielinor_protocol::appearance::FaceTarget::Entity {
                entity_type: 2,
                index: Some(1),
                fallback_angle: 0,
            },
        });
        let npc_face = face.map(|f| f.encode_npc().unwrap());
        let output = if npc {
            combat_updates::NpcUpdates {
                hitmarks,
                headbars,
                sequence,
                face: npc_face.as_deref(),
                spot_animations: None,
            }
            .encode()
        } else {
            gielinor_protocol::appearance::Updates {
                hitmarks,
                headbars,
                sequence,
                face,
                ..Default::default()
            }
            .encode()
            .unwrap()
        };
        assert_eq!(
            output
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>(),
            f[6],
            "case {case}"
        );
    }
}
#[test]
fn actual_rsprot_hitmarks_and_headbars_match_observer_selection_and_counts() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/combat-updates.expected"
    ))
    .expect("Run scripts/combat-updates-parity.ps1");
    assert_eq!(fixture.lines().count(), 4096);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let npc = f[0].starts_with('N');
        let observer = f[1].parse().unwrap();
        let target = f[2].parse().unwrap();
        let rows: Vec<Vec<i32>> = f[3]
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.split(',').map(|v| v.parse().unwrap()).collect())
            .collect();
        let output = if f[0].ends_with('H') {
            let hits: Vec<_> = rows
                .iter()
                .map(|a| Hitmark {
                    source_index: a[0],
                    source_type: a[1] as u16,
                    self_type: a[2] as u16,
                    other_type: a[3] as u16,
                    value: a[4] as u16,
                    delay: a[5] as u16,
                    limit: a[6] as u8,
                })
                .collect();
            combat_updates::hitmarks(npc, observer, target, &hits)
        } else {
            let bars: Vec<_> = rows
                .iter()
                .map(|a| Headbar {
                    source_index: a[0],
                    self_type: a[1] as u16,
                    other_type: a[2] as u16,
                    start_fill: a[3] as u8,
                    end_fill: a[4] as u8,
                    start_time: a[5] as u16,
                    end_time: a[6] as u16,
                })
                .collect();
            combat_updates::headbars(npc, observer, target, &bars)
        }
        .unwrap();
        assert_eq!(
            output
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>(),
            f[4],
            "case {case}"
        );
    }
}
