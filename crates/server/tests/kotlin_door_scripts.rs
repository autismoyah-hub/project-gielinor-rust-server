use gielinor_world::{
    doors::{self, Door, Kind},
    locations::Location,
    simulation::{World, WorldEvent},
    CoordGrid,
};
fn parse(s: &str) -> Location {
    let n: Vec<i32> = s.split(',').map(|v| v.parse().unwrap()).collect();
    let p = n[0];
    Location::new(
        CoordGrid::new((p >> 14) & 16383, p & 16383, (p >> 28) & 3).unwrap(),
        n[1] as u32,
        n[2] as u8,
        n[3] as u8,
    )
    .unwrap()
}
fn state(w: &World, p: CoordGrid) -> String {
    w.locations
        .all(p)
        .into_iter()
        .map(|l| {
            format!(
                "{},{},{},{}",
                (l.coords.level() << 28) | (l.coords.x() << 14) | l.coords.z(),
                l.id,
                l.shape,
                l.angle
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}
fn wire(w: &mut World, p: CoordGrid) -> String {
    let mut bytes = vec![];
    for event in w.drain_events() {
        if let WorldEvent::Location(event) = event {
            let q = event.location().coords;
            if q.x() >> 3 == p.x() >> 3 && q.z() >> 3 == p.z() >> 3 && q.level() == p.level() {
                bytes.extend(
                    gielinor_server::ground_sync::location(event)
                        .encode()
                        .unwrap(),
                );
            }
        }
    }
    bytes.iter().map(|v| format!("{v:02x}")).collect()
}
#[test]
fn actual_kotlin_door_scripts_match_mutations_sounds_and_duration_expiry() {
    let rows = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/door-scripts.expected"
    ))
    .expect("Run scripts/door-scripts-parity.ps1");
    assert_eq!(rows.lines().count(), 768);
    let kinds = [
        Kind::ClosedSingle,
        Kind::OpenedSingle,
        Kind::ClosedLeft,
        Kind::ClosedRight,
        Kind::OpenedLeft,
        Kind::OpenedRight,
    ];
    for (case, line) in rows.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let kind = f[0].parse::<usize>().unwrap();
        let loc = parse(f[1]);
        let mut w = World::default();
        w.locations.load(loc);
        for other in f[2].split(';').filter(|v| !v.is_empty()) {
            w.locations.load(parse(other));
        }
        let mut catalog = doors::Catalog::new();
        for id in 1..=12 {
            let kind = kinds[(id as usize - 1) % 6];
            catalog.insert(
                id,
                Door {
                    kind,
                    next: id + 20,
                    sound: if kind == Kind::OpenedSingle { 60 } else { 62 },
                },
            );
        }
        let started = doors::begin(
            loc,
            catalog[&loc.id],
            loc.coords.translate(5, 5, 0).unwrap(),
        )
        .unwrap();
        assert!(started.step_away.is_none());
        assert_eq!(started.sound.to_string(), f[3], "sound {case} {kind}");
        doors::finish(&mut w, &catalog, started.pending).unwrap();
        assert_eq!(state(&w, loc.coords), f[4], "immediate {case}");
        assert_eq!(wire(&mut w, loc.coords), f[5], "wire {case}");
        w.clock.0 = 499;
        w.tick().unwrap();
        assert_eq!(state(&w, loc.coords), f[6], "expired {case}");
        assert_eq!(wire(&mut w, loc.coords), f[7], "expiry wire {case}");
    }
}
