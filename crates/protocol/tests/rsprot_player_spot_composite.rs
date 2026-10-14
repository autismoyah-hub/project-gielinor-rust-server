use gielinor_protocol::{
    appearance::{Face, FaceTarget, Updates},
    combat_updates::{self, Headbar, Hitmark},
    spot_animation::Animation,
};
#[test]
fn actual_rsprot_writer_combines_spots_combat_and_facing_in_mask_order() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/player-spots-composite.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 512);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let bits: u8 = f[0].parse().unwrap();
        let observer = f[1].parse().unwrap();
        let target = f[2].parse().unwrap();
        let id = f[3].parse().unwrap();
        let delay: u8 = f[4].parse().unwrap();
        let hits = combat_updates::hitmarks(
            false,
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
            false,
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
        let spots = [
            Animation {
                slot: 7,
                id: 1336,
                delay: 0,
                height: 0,
            },
            Animation {
                slot: 1,
                id,
                delay: delay as u16,
                height: 200,
            },
            Animation {
                slot: 7,
                id: 65535,
                delay: 500,
                height: 255,
            },
        ];
        let bytes = Updates {
            spot_animations: Some(&spots),
            hitmarks: (bits & 1 != 0).then_some(&hits),
            headbars: (bits & 2 != 0).then_some(&bars),
            sequence: (bits & 4 != 0).then_some((id, delay)),
            face: (bits & 8 != 0).then_some(Face {
                walk_mode: 0,
                instant: false,
                target: FaceTarget::Entity {
                    entity_type: 2,
                    index: Some(1),
                    fallback_angle: 0,
                },
            }),
            ..Default::default()
        }
        .encode()
        .unwrap();
        assert_eq!(
            bytes.iter().map(|b| format!("{b:02x}")).collect::<String>(),
            f[5],
            "case {case}"
        );
    }
}
