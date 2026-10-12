use gielinor_world::{
    doors::{self, Door, Kind, Translation},
    locations::Location,
    CoordGrid,
};
#[test]
fn compiled_door_translations_and_single_door_step_away_match() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/doors.expected"
    ))
    .expect("Run scripts/doors-parity.ps1");
    assert_eq!(fixture.lines().count(), 4096);
    for (line_no, line) in fixture.lines().enumerate() {
        let f = line.split('|').collect::<Vec<_>>();
        let pos = CoordGrid::from_packed(f[0].parse().unwrap());
        let shape = f[1].parse().unwrap();
        let angle = f[2].parse().unwrap();
        for (index, mode) in [
            Translation::Open,
            Translation::OpenOpposite,
            Translation::Close,
            Translation::CloseOpposite,
        ]
        .into_iter()
        .enumerate()
        {
            let value = doors::translate(pos, shape, angle, mode)
                .map(|v| v.packed().to_string())
                .unwrap_or("ERR".into());
            assert_eq!(value, f[3 + index], "translation {line_no}/{index}");
        }
        for (index, kind) in [Kind::ClosedSingle, Kind::OpenedSingle]
            .into_iter()
            .enumerate()
        {
            let target = f[if index == 0 { 3 } else { 5 }];
            let value = if target == "ERR" {
                "ERR".into()
            } else {
                doors::begin(
                    Location::new(pos, 1, shape, angle).unwrap(),
                    Door {
                        kind,
                        next: 2,
                        sound: 5,
                    },
                    CoordGrid::from_packed(target.parse().unwrap()),
                )
                .map(|p| p.step_away.unwrap_or(CoordGrid::NULL).packed().to_string())
                .unwrap_or("ERR".into())
            };
            assert_eq!(value, f[7 + index], "step away {line_no}/{index}");
        }
    }
}
