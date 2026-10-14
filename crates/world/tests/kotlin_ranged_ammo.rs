use gielinor_world::ranged_ammo::*;
#[test]
fn compiled_ammunition_validation_matches_categories_bone_and_levels() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/ranged-ammo.expected"
    ))
    .expect("Run scripts/ranged-ammo-parity.ps1");
    assert_eq!(fixture.lines().count(), 4096);
    let categories = Categories {
        arrows: 10,
        dragon_arrow: 11,
        bolts: 12,
        javelins: 13,
    };
    for (case, line) in fixture.lines().enumerate() {
        let (input, expected) = line.split_once('|').unwrap();
        let a: Vec<i32> = input.split(',').map(|x| x.parse().unwrap()).collect();
        let kind = [Kind::Bow, Kind::Crossbow, Kind::Ballista][a[0] as usize];
        let weapon = Weapon {
            kind,
            required: (a[1] >= 0).then_some(a[1]),
            level: a[3],
            bone: a[5],
        };
        let ammo = Ammo {
            category: a[2],
            level: a[4],
            bone: a[6],
        };
        assert_eq!(
            format!("{:?}", categories.validate(weapon, Some(ammo))),
            expected,
            "case {case}"
        );
    }
}
