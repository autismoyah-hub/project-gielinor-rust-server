use gielinor_cache::{
    definitions::{ItemType, Param},
    game::Definitions,
    parameters::ParameterType,
    symbols::Symbols,
};
use gielinor_world::inventory::{Inventory, Item, StackType};
use std::collections::BTreeMap;
#[test]
fn compiled_melee_collector_matches_cache_aliases_slots_sets_and_weapon_types() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/melee-equipment.expected"
    ))
    .expect("Run scripts/melee-equipment-parity.ps1");
    let mut objects = Symbols::new();
    let mut defs = Definitions {
        inventories: BTreeMap::new(),
        locations: BTreeMap::new(),
        npcs: BTreeMap::new(),
        items: BTreeMap::new(),
        varbits: BTreeMap::new(),
        varps: BTreeMap::new(),
        stats: BTreeMap::new(),
    };
    for line in fixture.lines().filter(|l| l.starts_with("I|")) {
        let f: Vec<_> = line.split('|').collect();
        let id = f[1].parse::<i32>().unwrap();
        objects.insert(f[2].into(), id);
        let mut item = ItemType::decode(&[0]).unwrap();
        item.category = id % 7;
        for key in 0..4 {
            item.params.insert(
                key,
                Param::Integer(if id % 11 == key as i32 { id % 3 - 1 } else { 0 }),
            );
        }
        defs.items.insert(id as u32, item);
    }
    let params: Symbols = [
        "blackmask",
        "slayer_helm",
        "blackmask_imbued",
        "slayer_helm_imbued",
    ]
    .into_iter()
    .enumerate()
    .map(|(i, n)| (n.to_owned(), i as i32))
    .collect();
    let defaults: BTreeMap<_, _> = (0..4)
        .map(|i| {
            (
                i,
                ParameterType::decode(&[1, b'i', 2, 0, 0, 0, 0, 0]).unwrap(),
            )
        })
        .collect();
    let categories: Symbols = [("halberd".into(), 1), ("spear".into(), 2)].into();
    let config = gielinor_server::melee_equipment_config::resolve(
        &objects,
        &params,
        &categories,
        &defaults,
        &defs,
    )
    .unwrap();
    let mut cases = 0;
    for line in fixture.lines().filter(|l| l.starts_with("C|")) {
        let f: Vec<_> = line.split('|').collect();
        let mut worn = Inventory::new(14, StackType::Normal, false);
        for (slot, id) in f[1]
            .split(',')
            .map(|s| s.parse::<i32>().unwrap())
            .enumerate()
        {
            if id >= 0 {
                worn.restore_slot(
                    slot,
                    Some(Item {
                        id,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
        }
        let kind = f[2].parse::<u8>().unwrap();
        let skull = f[3].parse().unwrap();
        let souls = f[4].parse().unwrap();
        assert_eq!(
            config.collect(&worn, (kind != 0).then_some(kind), skull),
            f[5].parse::<u64>().unwrap(),
            "attributes {cases}"
        );
        assert_eq!(
            config.void_multiplier(&worn),
            f[6].parse::<f64>().unwrap(),
            "void {cases}"
        );
        assert_eq!(
            config.soul_multiplier(&worn, souls),
            f[7].parse::<f64>().unwrap(),
            "souls {cases}"
        );
        cases += 1;
    }
    assert_eq!(cases, 4096);
}
