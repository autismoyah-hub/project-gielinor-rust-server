//! Independent public transaction checks at the vendor contract's original SHA.
use gielinor_world::{
    inventory::{Catalog, Inventory, Item, StackType},
    skillcape_vendor::{Config, Error, Kind},
    stats::Stats,
};
fn config(kind: Kind) -> Config {
    let (skill, npc, capes, hood) = if kind == Kind::Crafting {
        (12, 5810, [9780, 9781], 9782)
    } else {
        (7, 2658, [9801, 9802], 9803)
    };
    Config {
        kind,
        npc,
        skill,
        coins: 995,
        capes,
        hood,
        active_stats: 0x007fffff,
    }
}
fn bag(coin: usize, money: i32, free: usize) -> Inventory {
    let mut inv = Inventory::new(28, StackType::Normal, false);
    for s in 0..28 {
        let offset = (s + 28 - coin) % 28;
        if offset > 0 && offset <= free {
            continue;
        }
        inv.restore_slot(
            s,
            Some(Item {
                id: if s == coin { 995 } else { 1931 },
                count: if s == coin { money } else { 1 },
                vars: if s == coin { 0 } else { 7 },
            }),
        )
        .unwrap();
    }
    inv
}
fn stock(inv: &Inventory, id: i32) -> i32 {
    inv.slots()
        .iter()
        .flatten()
        .filter(|i| i.id == id)
        .map(|i| i.count)
        .sum()
}
#[test]
fn exact_payment_predebit_space_and_trim_uses_only_released_base_levels() {
    let mut cases = 0;
    for kind in [Kind::Crafting, Kind::Cooking] {
        let c = config(kind);
        for coin in 0..28 {
            for money in [99000, 99001, 2_000_000_000] {
                for other in 0..5 {
                    for member in [false, true] {
                        for base in [98, 99] {
                            let inv = bag(coin, money, 2);
                            let worn = Inventory::new(14, StackType::Normal, false);
                            let mut items = Catalog::default();
                            items.stackable.insert(995);
                            let mut stats = Stats::default();
                            stats.set_base(c.skill, base);
                            stats.set_current(c.skill, 99);
                            match other {
                                1 => stats.set_base(0, 99),
                                2 => stats.set_base(23, 99),
                                3 => stats.set_current(0, 99),
                                4 => stats.set_base(22, 99),
                                _ => {}
                            }
                            let before = (inv.clone(), worn.clone(), stats.clone());
                            let out = c.prepare_purchase(&inv, &worn, &items, &stats, member);
                            if !member {
                                assert!(matches!(out, Err(Error::Members)));
                            } else if base < 99 {
                                assert!(matches!(out, Err(Error::Level)));
                            } else {
                                let p = out.unwrap();
                                let cape = c.capes[usize::from(other == 1 || other == 4)];
                                assert_eq!(p.output, cape);
                                assert_eq!(stock(&p.inventory, 995), money - 99000);
                                assert_eq!(stock(&p.inventory, cape), 1);
                                assert_eq!(stock(&p.inventory, c.hood), 1);
                                assert_eq!(stock(&p.inventory, 1931), 25);
                                for s in 0..28 {
                                    if let Some(i) = inv.get(s) {
                                        if i.id == 1931 {
                                            assert_eq!(p.inventory.get(s), Some(i));
                                        }
                                    }
                                }
                            }
                            assert_eq!((inv, worn, stats), before);
                            cases += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(cases, 3360);
    println!("PG_VENDOR_MATRIX {cases} bothvendors all28coinslots threepayments fivebase/current/releasedstatpatterns member/F2P base98/99 exactdebit twooutputs unrelatedvars preserved PASS");
}
#[test]
fn exact_coins_do_not_create_predebit_space_and_no_room_means_no_partial_plan() {
    let mut cases = 0;
    for kind in [Kind::Crafting, Kind::Cooking] {
        let c = config(kind);
        for coin in 0..28 {
            for money in [98999, 99000, 99001] {
                for free in [0, 1] {
                    let inv = bag(coin, money, free);
                    let worn = Inventory::new(14, StackType::Normal, false);
                    let mut items = Catalog::default();
                    items.stackable.insert(995);
                    let mut stats = Stats::default();
                    stats.set_base(c.skill, 99);
                    let before = inv.clone();
                    let result = c.prepare_purchase(&inv, &worn, &items, &stats, true);
                    assert!(if money < 99000 {
                        matches!(result, Err(Error::Money))
                    } else {
                        matches!(result, Err(Error::Space))
                    });
                    assert_eq!(inv, before);
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 336);
    println!("PG_VENDOR_SPACE {cases} underfunded/full/onefree evenexactpayment predebitrefusal noPartialPlan PASS");
}
