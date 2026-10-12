use gielinor_world::{
    loc_timers::Timers,
    locations::{Changed, Location, Locations},
    tick::MapClock,
    CoordGrid,
};
fn parse(value: &str) -> Location {
    let n = value
        .split(',')
        .map(|n| n.parse::<i32>().unwrap())
        .collect::<Vec<_>>();
    Location::new(
        CoordGrid::new(n[0], n[1], n[2]).unwrap(),
        n[3] as u32,
        n[4] as u8,
        n[5] as u8,
    )
    .unwrap()
}
fn list(locs: impl IntoIterator<Item = Location>) -> String {
    locs.into_iter()
        .map(|l| {
            format!(
                "{},{},{},{},{},{}",
                l.coords.x(),
                l.coords.z(),
                l.coords.level(),
                l.id,
                l.shape,
                l.angle
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}
#[test]
fn actual_repository_timer_replacement_reversion_and_clock_saturation_match() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/location-timers.expected"
    ))
    .expect("Run scripts/location-timers-parity.ps1");
    assert_eq!(fixture.lines().count(), 2048);
    let mut current = String::new();
    let mut locations = Locations::default();
    let mut timers = Timers::default();
    for (line_no, line) in fixture.lines().enumerate() {
        let f = line.split('|').collect::<Vec<_>>();
        let loc = parse(f[4]);
        if current != f[0] {
            current = f[0].into();
            locations = Locations::default();
            timers = Timers::default();
            for i in 0..8 {
                locations.load(
                    Location::new(
                        CoordGrid::new(3200 + i, 3200, loc.coords.level()).unwrap(),
                        i as u32 + 1,
                        10,
                        i as u8 % 4,
                    )
                    .unwrap(),
                );
            }
        }
        let clock = MapClock(f[1].parse().unwrap());
        let mut updates = Vec::new();
        for (due, add) in timers.due(clock) {
            let (_, mutation) = if add {
                locations.add(due).unwrap()
            } else {
                locations.delete(due)
            };
            updates.extend(mutation.updates);
        }
        let add = f[2] == "1";
        let (changed, mutation) = if add {
            locations.add(loc).unwrap()
        } else {
            locations.delete(loc)
        };
        updates.extend(mutation.updates);
        timers.record(loc, add, changed, clock, f[3].parse().unwrap());
        assert_eq!(
            changed != Changed::Missing,
            f[5] == "1",
            "success {line_no}"
        );
        assert_eq!(
            list(locations.spawned(loc.coords)),
            f[6],
            "spawned {line_no}"
        );
        assert_eq!(list(locations.all(loc.coords)), f[7], "all {line_no}");
        let wire = updates
            .into_iter()
            .flat_map(|e| gielinor_server::ground_sync::location(e).encode().unwrap())
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        assert_eq!(wire, f[8], "updates {line_no}");
    }
}
