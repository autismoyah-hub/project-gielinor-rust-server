use gielinor_cache::{
    definitions::{ItemType, Param, StatType},
    game::Definitions,
};
use gielinor_world::{
    equipment,
    inventory::{Catalog, Inventory, Item, StackType},
    stats::Stats,
};
use std::collections::BTreeMap;
#[test]
fn compiled_kotlin_equipment_requirements_and_messages() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/equipment-requirements.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 2048);
    let symbols = [
        "statreq1_skill",
        "statreq1_level",
        "statreq2_skill",
        "statreq2_level",
        "statreq_failmessage1",
        "statreq_failmessage2",
    ]
    .into_iter()
    .enumerate()
    .map(|(i, s)| (s.to_owned(), i as i32 + 1))
    .collect();
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('\t').collect();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let mut item = ItemType::decode(&[13, 0, 0]).unwrap();
        if n(2) & 1 != 0 {
            item.params.insert(1, Param::Integer(0));
            item.params.insert(2, Param::Integer(n(3)));
        }
        if n(2) & 2 != 0 {
            item.params.insert(3, Param::Integer(1));
            item.params.insert(4, Param::Integer(n(4)));
        }
        if n(5) & 1 != 0 {
            item.params
                .insert(5, Param::String("Custom level warning.".into()));
        }
        if n(5) & 2 != 0 {
            item.params.insert(
                6,
                Param::String("Need {skill1}:{level1} + {skill2}:{level2}.".into()),
            );
        }
        let defs = Definitions {
            items: BTreeMap::from([(1, item)]),
            stats: ["Attack", "Defence"]
                .into_iter()
                .enumerate()
                .map(|(i, name)| {
                    let mut stat = StatType::decode(&[0]).unwrap();
                    stat.name = name.into();
                    (i as u32, stat)
                })
                .collect(),
            inventories: BTreeMap::new(),
            npcs: BTreeMap::new(),
            locations: BTreeMap::new(),
            varps: BTreeMap::new(),
            varbits: BTreeMap::new(),
        };
        let wear = gielinor_server::content::equipment(&symbols, &defs).unwrap();
        let mut stats = Stats::default();
        stats.set_base(0, n(0) as i8);
        stats.set_base(1, n(1) as i8);
        let mut held = Inventory::new(1, StackType::Normal, false);
        let mut worn = Inventory::new(15, StackType::Normal, false);
        held.restore_slot(
            0,
            Some(Item {
                id: 1,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        let (status, messages) =
            match equipment::equip(&mut held, &mut worn, 0, &Catalog::default(), &wear, &stats) {
                Ok(_) => ("Success", "-".into()),
                Err(equipment::Error::Requirements(missing)) => (
                    "StatRequirements",
                    gielinor_server::content::equipment_requirement_messages(
                        &defs,
                        1,
                        &missing,
                        [5, 6],
                    )
                    .unwrap()
                    .join("|"),
                ),
                Err(e) => panic!("{e:?}"),
            };
        assert_eq!(
            (status, messages.as_str()),
            (f[6], f[7]),
            "case {case}: {line}"
        );
    }
}
