use gielinor_world::{
    inventory::{Catalog, Inventory, Item, StackType, Template},
    skillcape_vendor::{Config, Error, Kind},
    stats::Stats,
};
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
#[test]
fn already_owned_crafting_and_cooking_policies_differ_without_mutating_old_capes() {
    let mut cases = 0;
    for kind in [Kind::Crafting, Kind::Cooking] {
        for cape in 0..2 {
            for on_body in [false, true] {
                for hood_place in 0..3 {
                    let (c, mut inv, mut worn, cat, s) = setup(kind);
                    if on_body {
                        worn.restore_slot(
                            1,
                            Some(Item {
                                id: c.capes[cape],
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    } else {
                        inv.restore_slot(
                            0,
                            Some(Item {
                                id: c.capes[cape],
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    }
                    if hood_place == 1 {
                        inv.restore_slot(
                            1,
                            Some(Item {
                                id: c.hood,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    } else if hood_place == 2 {
                        worn.restore_slot(
                            0,
                            Some(Item {
                                id: c.hood,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    }
                    let before = (inv.clone(), worn.clone(), s.clone());
                    let paid = c.prepare_purchase(&inv, &worn, &cat, &s, true);
                    if kind == Kind::Crafting {
                        assert!(matches!(paid, Err(Error::OwnCape)));
                    } else {
                        let p = paid.unwrap();
                        assert_eq!(units(&p.inventory, 995), 1);
                        assert_eq!(
                            units(&p.inventory, c.capes[cape]),
                            units(&inv, c.capes[cape]) + i32::from(cape == 0)
                        );
                        assert_eq!(units(&p.inventory, c.capes[0]), units(&inv, c.capes[0]) + 1);
                        assert_eq!(units(&p.inventory, c.hood), units(&inv, c.hood) + 1);
                        assert_eq!(p.inventory.get(27), inv.get(27));
                    }
                    let hood = c.prepare_hood(&inv, &worn, &cat, &s, true);
                    if kind == Kind::Crafting && hood_place > 0 {
                        assert!(matches!(hood, Err(Error::OwnHood)));
                    } else {
                        let p = hood.unwrap();
                        assert_eq!(units(&p.inventory, 995), 99001);
                        assert_eq!(units(&p.inventory, c.hood), units(&inv, c.hood) + 1);
                        for id in c.capes {
                            assert_eq!(units(&p.inventory, id), units(&inv, id));
                        }
                    }
                    assert_eq!((inv, worn, s), before);
                    cases += 2;
                }
            }
        }
    }
    assert_eq!(cases, 48);
    println!("PG_VENDOR_OWNED48 bothcapes held/worn/hoodnone-held-worn Craftrefusespaidspare/existinghood Cookpaidspare/freeadditionalhood preservesoldcape coinsStats PASS");
}
#[test]
fn invalid_catalog_config_and_currency_never_prepare_partial_outputs() {
    let mut cases = 0;
    for kind in [Kind::Crafting, Kind::Cooking] {
        for id_index in 0..4 {
            for bad in 0..5 {
                let (c, inv, worn, mut cat, s) = setup(kind);
                let id = [c.coins, c.capes[0], c.capes[1], c.hood][id_index];
                match bad {
                    0 => {
                        cat.dummy.insert(id);
                    }
                    1 => {
                        if id == 995 {
                            cat.stackable.remove(&id);
                        } else {
                            cat.stackable.insert(id);
                        }
                    }
                    2 => {
                        cat.notes.insert(
                            id,
                            Template {
                                link: 1931,
                                template: 799,
                            },
                        );
                    }
                    3 => {
                        cat.placeholders.insert(
                            id,
                            Template {
                                link: 1931,
                                template: 14401,
                            },
                        );
                    }
                    _ => {
                        cat.transforms.insert(
                            id,
                            Template {
                                link: 1931,
                                template: 799,
                            },
                        );
                    }
                }
                let before = inv.clone();
                assert!(matches!(
                    c.prepare_purchase(&inv, &worn, &cat, &s, true),
                    Err(Error::Policy)
                ));
                assert_eq!(inv, before);
                cases += 1;
            }
        }
        for bad in 0..12 {
            let (mut c, mut inv, mut worn, cat, s) = setup(kind);
            match bad {
                0 => c.npc = 0,
                1 => c.skill = 24,
                2 => c.active_stats = 1 << 24,
                3 => c.active_stats = 0,
                4 => c.capes[1] = c.hood,
                5 => c.coins = -1,
                6 => inv.stack = StackType::Always,
                7 => inv.placeholders = true,
                8 => worn.placeholders = true,
                9 => inv
                    .restore_slot(
                        8,
                        Some(Item {
                            id: 995,
                            count: 0,
                            vars: 0,
                        }),
                    )
                    .unwrap(),
                10 => inv
                    .restore_slot(
                        8,
                        Some(Item {
                            id: 995,
                            count: 99001,
                            vars: 1,
                        }),
                    )
                    .unwrap(),
                _ => inv
                    .restore_slot(
                        9,
                        Some(Item {
                            id: 995,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap(),
            }
            let before = inv.clone();
            assert!(
                matches!(
                    c.prepare_purchase(&inv, &worn, &cat, &s, true),
                    Err(Error::Policy)
                ),
                "kind{kind:?}bad{bad}"
            );
            assert_eq!(inv, before);
            cases += 1;
        }
    }
    assert_eq!(cases, 64);
    println!("PG_VENDOR_GUARD64 bothvendors fourtypes xfivebadcatalog plus12config/bag/coinfaults refusePolicy noPartialPlan PASS");
}
