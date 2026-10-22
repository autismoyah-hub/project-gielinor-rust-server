use super::*;
use gielinor_world::{
    CoordGrid,
    inventory::Item,
    simulation::{Action, ActorId, PlayerData, World},
    stats::Stats,
};

// Controlled fixtures use independently verified target IDs/flags, not donor241 IDs.
const PROFILES: [(&str, i32, &str, i32, bool, bool); 10] = [
    (
        "pack_slayer_broad_bolt_unfinished",
        11887,
        "slayer_broad_bolt_unfinished",
        11876,
        false,
        true,
    ),
    (
        "pack_slayer_broad_arrowhead",
        11885,
        "slayer_broad_arrowhead",
        11874,
        false,
        true,
    ),
    ("pack_softclay", 12009, "softclay", 1761, true, false),
    ("pack_amylase", 12641, "amylase", 12640, false, true),
    ("pack_eye_newt", 12859, "eye_of_newt", 221, true, false),
    (
        "pack_box_trap",
        12742,
        "hunting_box_trap",
        10008,
        false,
        true,
    ),
    (
        "pack_ojibway_bird_snare",
        12740,
        "hunting_ojibway_bird_snare",
        10006,
        false,
        true,
    ),
    (
        "pack_magic_imp_box",
        12744,
        "magic_imp_box",
        10025,
        false,
        true,
    ),
    (
        "pack_bone_bolt",
        13193,
        "dttd_bone_crossbow_bolt",
        8882,
        false,
        true,
    ),
    (
        "piscarilius_pack_sandworms",
        13432,
        "piscarilius_sandworms",
        13431,
        false,
        false,
    ),
];

fn definitions() -> (Symbols, Definitions) {
    let mut s = Symbols::new();
    let mut d = Definitions {
        inventories: Default::default(),
        locations: Default::default(),
        npcs: Default::default(),
        items: Default::default(),
        varbits: Default::default(),
        varps: Default::default(),
        stats: Default::default(),
    };
    for (name, pack, output_name, output, noted, member) in PROFILES {
        s.insert(name.into(), pack);
        s.insert(output_name.into(), output);
        let mut ty = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
        ty.members = member;
        ty.inventory_ops[0] = Some("Open".into());
        d.items.insert(pack as u32, ty);
        let mut ty = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
        ty.stackable = !noted;
        if noted {
            ty.cert.link = output + 1;
            let mut note = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            note.stackable = true;
            note.cert.template = 799;
            note.cert.link = output;
            d.items.insert((output + 1) as u32, note);
        }
        d.items.insert(output as u32, ty);
    }
    (s, d)
}
fn world(pack: i32, slot: usize, member: bool) -> (World, ActorId) {
    let (s, d) = definitions();
    let mut w = World::default();
    w.item_packs = item_packs(&s, &d).unwrap();
    w.member_item_packs = item_pack_members(&w.item_packs, &d).unwrap();
    w.object_types = d.inventory_catalog();
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
    let a = w.actors().next().unwrap();
    w.set_membership(a, member).unwrap();
    (w, a)
}
fn total(w: &World, a: ActorId, item: i32) -> i32 {
    w.data(a)
        .unwrap()
        .inventory
        .slots()
        .iter()
        .flatten()
        .filter(|i| i.id == item)
        .map(|i| i.count)
        .sum()
}

#[test]
fn remaining_packs_bind_exact_stack_or_note_policy_and_fail_malformed_notes() {
    let (s, d) = definitions();
    let rules = item_packs(&s, &d).unwrap();
    assert_eq!(rules.len(), 10);
    for (name, pack, output_name, output, noted, member) in PROFILES {
        let rule = rules[&pack];
        assert_eq!((rule.output, rule.count, rule.noted), (output, 100, noted));
        assert_eq!(
            item_pack_members(&rules, &d).unwrap().contains(&pack),
            member
        );
        let mut optional = s.clone();
        optional.remove(name);
        assert_eq!(item_packs(&optional, &d).unwrap().len(), 9);
        let mut missing = s.clone();
        missing.remove(output_name);
        assert!(item_packs(&missing, &d).is_err());
        let (_, mut bad) = definitions();
        bad.items.get_mut(&(pack as u32)).unwrap().inventory_ops[0] = Some("Use".into());
        assert!(item_packs(&s, &bad).is_err());
        if noted {
            let (_, mut bad) = definitions();
            bad.items.get_mut(&((output + 1) as u32)).unwrap().cert.link = 999;
            assert!(item_packs(&s, &bad).is_err());
        }
    }
    // Donor241 shark_lure_pack is not present in the supported239.1 symbols.
    let mut future = s.clone();
    future.insert("shark_lure_pack".into(), 34065);
    assert_eq!(item_packs(&future, &d).unwrap().len(), 10);
}

#[test]
fn remaining_packs_world_full_inventory_selected_repeat_and_member_matrix() {
    let mut cases = 0;
    for (_, pack, _, output, noted, required_member) in PROFILES {
        for slot in [0, 13, 27] {
            for member in [false, true] {
                let (mut w, a) = world(pack, slot, member);
                let before = w.data(a).unwrap().clone();
                w.submit(a, Action::OpenItemPack { slot, item: pack })
                    .unwrap();
                for _ in 0..8 {
                    w.tick().unwrap();
                }
                let allowed = member || !required_member;
                let reward = output + i32::from(noted);
                assert_eq!(total(&w, a, pack), if allowed { 0 } else { 2 });
                assert_eq!(total(&w, a, reward), if allowed { 200 } else { 0 });
                assert_eq!(w.data(a).unwrap().stats, before.stats);
                assert_eq!(total(&w, a, 1511), 26);
                if !allowed {
                    assert_eq!(
                        w.data(a).unwrap().inventory.slots(),
                        before.inventory.slots()
                    );
                }
                if noted {
                    assert_eq!(total(&w, a, output), 0);
                }
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 60);
}

#[test]
fn remaining_packs_overflow_stale_slot_and_zero_life_are_atomic() {
    let mut cases = 0;
    for (_, pack, _, output, noted, _) in PROFILES {
        for mode in 0..3 {
            let (mut w, a) = world(pack, 13, true);
            if mode == 0 {
                w.data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        0,
                        Some(Item {
                            id: output + i32::from(noted),
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
    assert_eq!(cases, 30);
}
