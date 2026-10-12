use gielinor_protocol::zone::Object;
use gielinor_world::{
    locations::{Changed, Location, Locations, Mutation, DELETED_ID},
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
fn describe(loc: Location) -> String {
    format!(
        "{},{},{},{},{},{}",
        loc.coords.x(),
        loc.coords.z(),
        loc.coords.level(),
        loc.id,
        loc.shape,
        loc.angle
    )
}
fn list(locs: impl IntoIterator<Item = Location>) -> String {
    locs.into_iter().map(describe).collect::<Vec<_>>().join(";")
}
fn encode(loc: Location, delete: bool) -> Object {
    let (x, z) = ((loc.coords.x() & 7) as u8, (loc.coords.z() & 7) as u8);
    if delete {
        Object::LocDelete {
            x,
            z,
            shape: loc.shape,
            angle: loc.angle,
        }
    } else {
        Object::LocAdd {
            id: loc.id as u16,
            x,
            z,
            shape: loc.shape,
            angle: loc.angle,
            ops: 255,
        }
    }
}
fn wire(prots: impl IntoIterator<Item = Object>) -> String {
    prots
        .into_iter()
        .flat_map(|p| p.encode().unwrap())
        .map(|b| format!("{b:02x}"))
        .collect()
}
#[test]
fn compiled_normal_location_registry_order_masking_restoration_and_wire_match() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/locations.expected"
    ))
    .expect("Run scripts/locations-parity.ps1");
    assert_eq!(fixture.lines().count(), 4096);
    let mut current = String::new();
    let mut registry = Locations::default();
    for (line_no, line) in fixture.lines().enumerate() {
        let f = line.split('|').collect::<Vec<_>>();
        if current != f[0] {
            current = f[0].into();
            registry = Locations::default();
        }
        let loc = parse(f[2]);
        let (result, mutation) = match f[1] {
            "0" => {
                registry.load(loc);
                ("load", Mutation::default())
            }
            op => {
                let (changed, mutation) = if op == "1" {
                    registry.add(loc).unwrap()
                } else {
                    registry.delete(loc)
                };
                (
                    match changed {
                        Changed::Map => "NormalMapLoc",
                        Changed::Spawned => "NormalSpawned",
                        Changed::Missing => "LocNotFound",
                    },
                    mutation,
                )
            }
        };
        assert_eq!(result, f[3], "result line {line_no}");
        assert_eq!(
            list(registry.spawned(loc.coords)),
            f[4],
            "spawned line {line_no}"
        );
        assert_eq!(list(registry.all(loc.coords)), f[5], "all line {line_no}");
        assert_eq!(
            registry
                .find_type(loc.coords, loc.id)
                .map(describe)
                .unwrap_or("n".into()),
            f[6],
            "type line {line_no}"
        );
        assert_eq!(
            registry
                .find_shape(loc.coords, loc.shape)
                .map(describe)
                .unwrap_or("n".into()),
            f[7],
            "shape line {line_no}"
        );
        assert_eq!(
            wire(
                mutation
                    .updates
                    .into_iter()
                    .map(gielinor_server::ground_sync::location)
            ),
            f[8],
            "updates line {line_no}"
        );
        assert_eq!(
            wire(
                registry
                    .spawned(loc.coords)
                    .map(|v| encode(v, v.id == DELETED_ID))
            ),
            f[9],
            "persistent line {line_no}"
        );
    }
}
