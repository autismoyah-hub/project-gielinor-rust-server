use gielinor_world::{
    drop_actions::{Kind, Pending, Stage},
    ground::Objects,
    ground_transfer::{drop_slot, Drop},
    inventory::{Catalog, Inventory, Item, StackType},
    tick::MapClock,
    CoordGrid,
};
fn inventory(text: &str) -> Inventory {
    let mut result = Inventory::new(8, StackType::Normal, false);
    for (slot, item) in text.split(';').enumerate() {
        if item != "." {
            let n: Vec<i32> = item.split(',').map(|v| v.parse().unwrap()).collect();
            result
                .restore_slot(
                    slot,
                    Some(Item {
                        id: n[0],
                        count: n[1],
                        vars: n[2],
                    }),
                )
                .unwrap();
        }
    }
    result
}
fn slots(inv: &Inventory) -> String {
    inv.slots()
        .iter()
        .map(|item| item.map_or(".".into(), |v| format!("{},{},{}", v.id, v.count, v.vars)))
        .collect::<Vec<_>>()
        .join(";")
}
#[test]
fn actual_held_drop_commits_match_slot_mutation_capacity_and_inventory_fallback() {
    let rows = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/held-drop.expected"
    ))
    .expect("Run scripts/held-drop-parity.ps1");
    assert_eq!(rows.lines().count(), 3072);
    let mut catalog = Catalog::default();
    catalog.stackable.insert(1);
    let coords = CoordGrid::new(3200, 3200, 0).unwrap();
    for (case, line) in rows.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let slot = f[1].parse().unwrap();
        let mut inv = inventory(f[4]);
        let mut objects = Objects::default();
        let kind = match f[0] {
            "drop" => Kind::Drop,
            "destroy" => Kind::Destroy,
            _ => Kind::Release,
        };
        let pending = Pending {
            slot,
            item: Item {
                id: f[2].parse().unwrap(),
                count: f[3].parse().unwrap(),
                vars: 0,
            },
            kind,
            stage: Stage::Ready,
        };
        let mut sound = "";
        let mut message = "";
        if kind == Kind::Drop {
            if drop_slot(
                &mut objects,
                &mut inv,
                &catalog,
                MapClock(0),
                Drop {
                    slot,
                    count: i32::MAX,
                    coords,
                    observer: 900,
                    duration: 200,
                    reveal: 100,
                },
            )
            .is_ok()
            {
                sound = "2739";
            }
        } else if pending.delete(&mut inv, &catalog).is_ok() && kind == Kind::Release {
            message = "Released.";
        }
        let ground = objects
            .at(coords)
            .map(|h| {
                let v = h.snapshot();
                format!("{},{}", v.item, v.count)
            })
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(slots(&inv), f[5], "inventory {case}");
        assert_eq!(ground, f[6], "ground {case}");
        assert_eq!(sound, f[7], "sound {case}");
        assert_eq!(message, f[8], "message {case}");
    }
}
