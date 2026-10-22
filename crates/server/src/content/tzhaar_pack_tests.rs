use super::*;
use gielinor_world::{
    CoordGrid,
    inventory::Item,
    simulation::{Action, PlayerData, World},
    stats::Stats,
};

const PROFILES: [(&str, i32, &str, i32); 4] = [
    ("pack_airrune_tzhaar", 21698, "airrune", 556),
    ("pack_waterrune_tzhaar", 21701, "waterrune", 555),
    ("pack_earthrune_tzhaar", 21704, "earthrune", 557),
    ("pack_firerune_tzhaar", 21707, "firerune", 554),
];

// Controlled definitions test binding/World behavior; real-cache evidence is separate.
fn definitions() -> (Symbols, Definitions) {
    let mut symbols = Symbols::new();
    let mut defs = Definitions {
        inventories: Default::default(),
        locations: Default::default(),
        npcs: Default::default(),
        items: Default::default(),
        varbits: Default::default(),
        varps: Default::default(),
        stats: Default::default(),
    };
    for (name, pack, output_name, output) in PROFILES {
        symbols.insert(name.into(), pack);
        symbols.insert(output_name.into(), output);
        let mut ty = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
        ty.members = true;
        ty.inventory_ops[0] = Some("Open".into());
        defs.items.insert(pack as u32, ty);
        let mut ty = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
        ty.stackable = true;
        defs.items.insert(output as u32, ty);
    }
    (symbols, defs)
}

fn world(pack: i32, slot: usize, member: bool) -> (World, gielinor_world::simulation::ActorId) {
    let (symbols, defs) = definitions();
    let mut w = World::default();
    w.item_packs = item_packs(&symbols, &defs).unwrap();
    w.member_item_packs = item_pack_members(&w.item_packs, &defs).unwrap();
    w.object_types = defs.inventory_catalog();
    w.collision.allocate(3200, 3200, 0);
    let mut data = PlayerData::new(
        1,
        "Pack fixture".into(),
        CoordGrid::new(3200, 3200, 0).unwrap(),
        Stats::default(),
    );
    for i in 0..28 {
        data.inventory
            .restore_slot(
                i,
                Some(Item {
                    id: 1511,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
    }
    for i in [slot, (slot + 1) % 28] {
        data.inventory
            .restore_slot(
                i,
                Some(Item {
                    id: pack,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
    }
    w.queue_login(1, data).unwrap();
    w.tick().unwrap();
    let actor = w.actors().next().unwrap();
    w.set_membership(actor, member).unwrap();
    (w, actor)
}

fn total(w: &World, a: gielinor_world::simulation::ActorId, item: i32) -> i32 {
    w.data(a)
        .unwrap()
        .inventory
        .slots()
        .iter()
        .flatten()
        .filter(|v| v.id == item)
        .map(|v| v.count)
        .sum()
}

#[test]
fn tzhaar_pack_bindings_validate_open_outputs_and_optional_profiles() {
    let (s, d) = definitions();
    let rules = item_packs(&s, &d).unwrap();
    assert_eq!(rules.len(), 4);
    assert_eq!(
        item_pack_members(&rules, &d).unwrap(),
        PROFILES.map(|p| p.1).into()
    );
    assert!(item_packs(&Symbols::new(), &d).unwrap().is_empty());
    for (name, pack, output_name, output) in PROFILES {
        let rule = rules[&pack];
        assert_eq!((rule.output, rule.count, rule.noted), (output, 100, false));
        let (_, mut bad) = definitions();
        bad.items.get_mut(&(pack as u32)).unwrap().inventory_ops[0] = Some("Drop".into());
        assert!(item_packs(&s, &bad).is_err());
        let (_, mut bad) = definitions();
        bad.items.get_mut(&(output as u32)).unwrap().stackable = false;
        assert!(item_packs(&s, &bad).is_err());
        let mut bad = s.clone();
        bad.remove(output_name);
        assert!(item_packs(&bad, &d).is_err());
        let mut optional = s.clone();
        optional.remove(name);
        assert_eq!(item_packs(&optional, &d).unwrap().len(), 3);
    }
}

#[test]
fn tzhaar_packs_open_selected_and_repeat_only_for_members_in_full_duplicate_inventory() {
    let mut cases = 0;
    for (_, pack, _, output) in PROFILES {
        for slot in [0, 13, 27] {
            for member in [false, true] {
                let (mut w, a) = world(pack, slot, member);
                let before = w.data(a).unwrap().clone();
                w.submit(a, Action::OpenItemPack { slot, item: pack })
                    .unwrap();
                for _ in 0..8 {
                    w.tick().unwrap();
                }
                assert_eq!(total(&w, a, pack), if member { 0 } else { 2 });
                assert_eq!(total(&w, a, output), if member { 200 } else { 0 });
                assert_eq!(w.data(a).unwrap().stats, before.stats);
                if !member {
                    assert_eq!(
                        w.data(a).unwrap().inventory.slots(),
                        before.inventory.slots()
                    );
                }
                assert_eq!(total(&w, a, 1511), 26);
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 24);
}

#[test]
fn tzhaar_packs_refuse_overflow_stale_selection_and_zero_life_without_consumption() {
    let mut cases = 0;
    for (_, pack, _, output) in PROFILES {
        for mode in 0..3 {
            let (mut w, a) = world(pack, 13, true);
            if mode == 0 {
                w.data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        0,
                        Some(Item {
                            id: output,
                            count: i32::MAX,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
            if mode == 2 {
                w.data_mut(a).unwrap().stats.set_current(3, 0);
            }
            let before = w.data(a).unwrap().clone();
            w.submit(
                a,
                Action::OpenItemPack {
                    slot: if mode == 1 { 0 } else { 13 },
                    item: pack,
                },
            )
            .unwrap();
            for _ in 0..8 {
                w.tick().unwrap();
            }
            assert_eq!(
                w.data(a).unwrap().inventory.slots(),
                before.inventory.slots()
            );
            assert_eq!(w.data(a).unwrap().stats, before.stats);
            cases += 1;
        }
    }
    assert_eq!(cases, 12);
}
