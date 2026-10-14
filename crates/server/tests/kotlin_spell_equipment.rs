use gielinor_cache::{
    definitions::{ItemType, Param},
    game::Definitions,
    parameters::ParameterType,
    symbols::Symbols,
};
use gielinor_world::inventory::{Inventory, Item, StackType};
use std::collections::BTreeMap;
#[test]
fn compiled_spell_collector_matches_aliases_slots_and_ordered_random_draws() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/spell-equipment.expected"
    ))
    .expect("Run scripts/spell-equipment-parity.ps1");
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
        for key in 0..2 {
            item.params.insert(
                key,
                Param::Integer(if id % 11 == key as i32 { id % 3 - 1 } else { 0 }),
            );
        }
        defs.items.insert(id as u32, item);
    }
    let params: Symbols = ["blackmask_imbued", "slayer_helm_imbued"]
        .into_iter()
        .enumerate()
        .map(|(i, n)| (n.to_owned(), i as i32))
        .collect();
    let defaults: BTreeMap<_, _> = (0..2)
        .map(|i| {
            (
                i,
                ParameterType::decode(&[1, b'i', 2, 0, 0, 0, 0, 0]).unwrap(),
            )
        })
        .collect();
    let config =
        gielinor_server::spell_equipment_config::resolve(&objects, &params, &defaults, &defs)
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
        let mut draws = Replay(
            f[10]
                .split(',')
                .filter(|v| !v.is_empty())
                .map(|v| v.parse().unwrap())
                .collect(),
        );
        let context = gielinor_world::spell_equipment::Context {
            spell: f[2].parse().unwrap(),
            standard_book: f[3].parse().unwrap(),
            sunfire: f[4].parse().unwrap(),
            skull: f[5].parse().unwrap(),
            mark_of_darkness: f[6].parse().unwrap(),
            charge_cooldown: f[7].parse().unwrap(),
        };
        assert_eq!(
            config.collect(&worn, context, &mut draws),
            f[8].parse::<u64>().unwrap(),
            "attributes {cases}"
        );
        assert!(draws.0.is_empty(), "unused original draws {cases}");
        assert_eq!(
            config.accuracy_void(&worn),
            f[9].parse::<f64>().unwrap(),
            "void {cases}"
        );
        cases += 1;
    }
    assert_eq!(cases, 8192);
}

struct Replay(std::collections::VecDeque<u32>);
impl gielinor_world::npc::RandomSource for Replay {
    fn below(&mut self, bound: u32) -> u32 {
        assert_eq!(bound, 4);
        self.0.pop_front().expect("Unexpected extra passive draw")
    }
}
