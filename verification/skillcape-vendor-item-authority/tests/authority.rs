use gielinor_world::{
    inventory::{Catalog, Inventory, Item, StackType, Template},
    skillcape_vendor::{Config, Error, Kind},
    stats::Stats,
};

#[test]
fn all_wear_slots_counts_and_vars_are_authoritative_for_current_capes_and_hoods() {
    let mut calls = 0;
    for kind in [Kind::Crafting, Kind::Cooking] {
        for identity in 0..3 {
            for place in 0..15 {
                for count in [0, 1, 2] {
                    for vars in [0, 7] {
                        let (c, mut inv, mut worn, cat, s) = setup(kind);
                        let id = [c.capes[0], c.capes[1], c.hood][identity];
                        let item = Some(Item { id, count, vars });
                        if place == 0 {
                            inv.restore_slot(6, item).unwrap();
                        } else {
                            worn.restore_slot(place - 1, item).unwrap();
                        }
                        let before = (inv.clone(), worn.clone(), s.clone());
                        let valid = count == 1
                            && vars == 0
                            && (place == 0 || place - 1 == if identity == 2 { 0 } else { 1 });
                        let paid = c.prepare_purchase(&inv, &worn, &cat, &s, true);
                        let hood = c.prepare_hood(&inv, &worn, &cat, &s, true);
                        if !valid {
                            assert!(
                                matches!(paid, Err(Error::Policy)),
                                "{kind:?} id{id} place{place} count{count} vars{vars}"
                            );
                            assert!(matches!(hood, Err(Error::Policy)));
                        } else {
                            if kind == Kind::Crafting && identity < 2 {
                                assert!(matches!(paid, Err(Error::OwnCape)));
                            } else {
                                let plan = paid.unwrap();
                                assert_eq!(units(&plan.inventory, 995), 1);
                                assert_eq!(
                                    units(&plan.inventory, c.capes[0]),
                                    units(&inv, c.capes[0]) + 1
                                );
                                assert_eq!(units(&plan.inventory, c.hood), units(&inv, c.hood) + 1);
                            }
                            if identity == 2 {
                                assert!(matches!(hood, Err(Error::NeedCape)));
                            } else {
                                let plan = hood.unwrap();
                                assert_eq!(units(&plan.inventory, 995), 99001);
                                assert_eq!(units(&plan.inventory, c.hood), 1);
                            }
                        }
                        assert_eq!((inv, worn, s), before);
                        calls += 2;
                    }
                }
            }
        }
    }
    assert_eq!(calls, 1080);
    println!("PG_VENDOR_ITEM_AUTHORITY1080 bothvendors threecapehoodidentities held/all14wornslots count0/1/2 vars0/7 purchase+hood originalimagesretained PASS");
}
fn setup(kind: Kind) -> (Config, Inventory, Inventory, Catalog, Stats) {
    let (skill, npc, capes, hood) = if kind == Kind::Crafting {
        (12, 5810, [9780, 9781], 9782)
    } else {
        (7, 2658, [9801, 9802], 9803)
    };
    let c = Config {
        kind,
        npc,
        skill,
        coins: 995,
        capes,
        hood,
        active_stats: 0x007fffff,
    };
    let mut inv = Inventory::new(28, StackType::Normal, false);
    inv.restore_slot(
        8,
        Some(Item {
            id: 995,
            count: 99001,
            vars: 0,
        }),
    )
    .unwrap();
    inv.restore_slot(
        27,
        Some(Item {
            id: 1931,
            count: 1,
            vars: 9,
        }),
    )
    .unwrap();
    let worn = Inventory::new(14, StackType::Normal, false);
    let mut cat = Catalog::default();
    cat.stackable.insert(995);
    let mut s = Stats::default();
    s.set_base(skill, 99);
    s.set_current(skill, 99);
    (c, inv, worn, cat, s)
}
fn units(inv: &Inventory, id: i32) -> i32 {
    inv.slots()
        .iter()
        .flatten()
        .filter(|v| v.id == id)
        .map(|v| v.count)
        .sum()
}
