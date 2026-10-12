use gielinor_world::{
    ground::{Error, Object, Objects, Scope},
    inventory::Catalog,
    tick::MapClock,
    CoordGrid,
};
#[test]
fn original_ground_registry_and_lifecycle() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/ground.expected"
    ))
    .expect("Run scripts/ground-parity.ps1 to generate original Kotlin fixtures");
    assert_eq!(fixture.lines().count(), 6000);
    let mut objects = Objects::default();
    let mut current = -1;
    let mut catalog = Catalog::default();
    catalog.stackable.insert(995);
    catalog.dummy.insert(3);
    let coords = CoordGrid::new(3200, 3200, 0).unwrap();
    for (line_no, line) in fixture.lines().enumerate() {
        let fields: Vec<_> = line.split_whitespace().collect();
        let n = |i: usize| fields[i].parse::<i32>().unwrap();
        if current != n(0) {
            objects = Objects::default();
            current = n(0);
        }
        let clock = MapClock(n(1));
        objects.process_delayed(clock, &catalog).unwrap();
        objects.process_durations(clock, &catalog).unwrap();
        let scope = match n(5) {
            0 => Scope::Temporary,
            1 => Scope::Permanent,
            _ => Scope::Private,
        };
        let status = match n(2) {
            0 | 1 => match objects.add(
                Object {
                    coords,
                    item: n(3) as u16,
                    count: n(4),
                    scope,
                    created: clock.0,
                    receiver: (n(6) >= 0).then_some(n(6) as i64),
                    owner: (n(7) >= 0).then_some(n(7) as i64),
                },
                &catalog,
                clock,
                n(8),
                n(9),
            ) {
                Ok(_) => "ok",
                Err(Error::DummyItem) => "dummy",
                Err(Error::NonStackableLimit) => "limit",
                Err(e) => panic!("{e:?}"),
            },
            2 => {
                let entry = objects.at(coords).nth(n(10) as usize).cloned();
                if entry.is_some_and(|e| objects.delete(&e, clock, n(8))) {
                    "deleted"
                } else {
                    "missing"
                }
            }
            _ => "tick",
        };
        assert_eq!(status, fields[11], "status at {line_no}: {line}");
        let state = objects
            .at(coords)
            .map(|e| {
                let o = e.snapshot();
                let scope = match o.scope {
                    Scope::Temporary => 0,
                    Scope::Permanent => 1,
                    Scope::Private => 2,
                };
                format!(
                    "{}:{}:{}:{}:{}:{}",
                    o.item,
                    o.count,
                    scope,
                    o.receiver.unwrap_or(-1),
                    o.owner.unwrap_or(-1),
                    o.created
                )
            })
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(
            if state.is_empty() { "." } else { &state },
            fields[12],
            "state at {line_no}: {line}"
        );
        objects.drain_events();
    }
}
