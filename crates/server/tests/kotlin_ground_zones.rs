use gielinor_server::ground_sync;
use gielinor_world::{
    ground::{Event, Object, Objects, Scope},
    inventory::Catalog,
    locations::{Event as LocEvent, Location, Locations, DELETED_ID},
    tick::MapClock,
    zones::{Change, Observer},
    CoordGrid,
};
fn coord(fields: &[&str]) -> CoordGrid {
    CoordGrid::new(
        fields[0].parse().unwrap(),
        fields[1].parse().unwrap(),
        fields[2].parse().unwrap(),
    )
    .unwrap()
}
fn receiver(value: &str) -> Option<i64> {
    if value == "n" {
        None
    } else {
        Some(value.parse().unwrap())
    }
}
fn object(fields: &[&str]) -> Object {
    Object {
        coords: coord(fields),
        item: fields[3].parse().unwrap(),
        count: fields[4].parse().unwrap(),
        scope: match fields[5] {
            "0" => Scope::Temporary,
            "1" => Scope::Permanent,
            _ => Scope::Private,
        },
        created: 0,
        receiver: receiver(fields[6]),
        owner: None,
    }
}
#[test]
fn actual_kotlin_zone_processor_matches_visibility_order_and_wire_packets() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/ground-zones.expected"
    ))
    .expect("Run scripts/ground-zones-parity.ps1");
    assert_eq!(fixture.lines().count(), 2048);
    let mut previous = String::new();
    let mut observer = Observer::default();
    for (line_no, line) in fixture.lines().enumerate() {
        let fields = line.split('|').collect::<Vec<_>>();
        if previous != fields[0] {
            previous = fields[0].into();
            observer = Observer::default();
        }
        let pos = coord(&fields[1].split(',').collect::<Vec<_>>());
        let base = coord(&fields[2].split(',').collect::<Vec<_>>());
        let mut objects = Objects::default();
        let mut catalog = Catalog::default();
        catalog.stackable.extend(100..124);
        for item in fields[4].split(';') {
            objects
                .add(
                    object(&item.split(',').collect::<Vec<_>>()),
                    &catalog,
                    MapClock(0),
                    i32::MAX,
                    i32::MAX,
                )
                .unwrap();
        }
        let events = fields[5]
            .split(';')
            .enumerate()
            .map(|(id, entry)| {
                let e = entry.split(',').collect::<Vec<_>>();
                let obj = object(&e[1..]);
                let id = id as u64;
                match e[0] {
                    "0" => Event::Added(id, obj),
                    "1" => Event::Deleted(id, obj),
                    "2" => Event::Count(id, obj, e[8].parse().unwrap()),
                    _ => Event::Revealed(id, obj),
                }
            })
            .collect::<Vec<_>>();
        let cycle = line_no % 8;
        let mut locations = Locations::default();
        let mut changes = Vec::new();
        let loc = Location::new(pos, 1000 + cycle as u32, 10, (cycle % 4) as u8).unwrap();
        if pos.x() < 6432 {
            locations
                .add(Location {
                    id: if cycle % 3 == 0 { DELETED_ID } else { loc.id },
                    ..loc
                })
                .unwrap();
            changes.push(Change::Location(LocEvent::Add(loc)));
        }
        changes.extend(events.into_iter().map(Change::Ground));
        if pos.x() < 6432 {
            changes.push(Change::Location(LocEvent::Animate {
                location: loc,
                sequence: cycle as u16 * 1000,
            }));
            changes.push(Change::Location(LocEvent::Delete(loc)));
        }
        let emissions = observer.cycle_all(
            pos,
            base,
            receiver(fields[3]),
            &objects,
            &locations,
            &changes,
        );
        let visible = observer
            .visible()
            .iter()
            .map(|c| format!("{},{},{}", c.x(), c.z(), c.level()))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(visible, fields[6], "visible line {line_no}");
        if fields[7] == "ERR" {
            assert!(emissions.is_err(), "error line {line_no}");
            continue;
        }
        let emissions = emissions.unwrap();
        let packets = ground_sync::encode(base, &emissions)
            .unwrap()
            .iter()
            .map(|p| {
                format!(
                    "{}:{}",
                    p.opcode,
                    p.payload
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(packets, fields[7], "packets line {line_no}");
    }
}
