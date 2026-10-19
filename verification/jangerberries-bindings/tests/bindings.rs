//! Actual239.1 cache definitions and isolated malformed copies at exact145.
use gielinor_cache::{
    definitions::{ItemType, StatType},
    game::{Definitions, GameCache},
    symbols,
};
use gielinor_server::content::jangerberries;
use std::{collections::BTreeMap, path::Path};
fn copy(d: &Definitions) -> Definitions {
    Definitions {
        inventories: BTreeMap::new(),
        locations: BTreeMap::new(),
        npcs: BTreeMap::new(),
        items: d.items.clone(),
        varbits: BTreeMap::new(),
        varps: BTreeMap::new(),
        stats: d.stats.clone(),
    }
}
#[test]
fn actual_member_berry_four_stats_and_atomic_registration_faults() {
    let names = std::env::var("PG_SYMBOLS").unwrap();
    let names = |kind| symbols::load(&[Path::new(&names)], kind, None).unwrap();
    let obj = names("obj");
    let stat = names("stat");
    let path = std::env::var("PG_GAME_CACHE").unwrap();
    let mut cache = GameCache::open(Path::new(&path), 0xff_ffff).unwrap();
    let items = cache.files(2, 10, None).unwrap().unwrap();
    let stats = cache.files(2, 200, None).unwrap().unwrap();
    let mut d = Definitions {
        inventories: BTreeMap::new(),
        locations: BTreeMap::new(),
        npcs: BTreeMap::new(),
        items: BTreeMap::new(),
        varbits: BTreeMap::new(),
        varps: BTreeMap::new(),
        stats: BTreeMap::new(),
    };
    d.items.insert(247, ItemType::decode(&items[&247]).unwrap());
    for id in [0, 1, 2, 5] {
        d.stats.insert(id, StatType::decode(&stats[&id]).unwrap());
    }
    let mut foods = BTreeMap::new();
    let p = jangerberries(&obj, &stat, &d, &mut foods).unwrap();
    assert_eq!(p.item, 247);
    assert_eq!(
        p.boosts
            .iter()
            .map(|r| (r.skill, r.amount))
            .collect::<Vec<_>>(),
        [(0, 2), (2, 1)]
    );
    assert_eq!(
        p.subtractions
            .iter()
            .map(|r| (r.skill, r.amount))
            .collect::<Vec<_>>(),
        [(1, 1)]
    );
    assert_eq!(
        p.healing
            .iter()
            .map(|r| (r.skill, r.amount))
            .collect::<Vec<_>>(),
        [(5, 1)]
    );
    let f = foods[&247].clone();
    assert_eq!((f.heal, f.delay, f.replacement), (2, 3, None));
    assert_eq!(f.message, "You eat the jangerberries.");
    let before = format!("{foods:?}");
    assert!(jangerberries(&obj, &stat, &d, &mut foods).is_err());
    assert_eq!(format!("{foods:?}"), before);
    let mut n = 1;
    for mode in 0..11 {
        let mut q = copy(&d);
        let mut o = obj.clone();
        match mode {
            0 => {
                q.items.remove(&247);
            }
            1 => q.items.get_mut(&247).unwrap().name = "Wrong".into(),
            2 => q.items.get_mut(&247).unwrap().members = false,
            3 => q.items.get_mut(&247).unwrap().stackable = true,
            4 => q.items.get_mut(&247).unwrap().dummy = 1,
            5 => q.items.get_mut(&247).unwrap().cert.template = 799,
            6 => q.items.get_mut(&247).unwrap().placeholder.template = 14401,
            7 => q.items.get_mut(&247).unwrap().transform.template = 1,
            8 => q.items.get_mut(&247).unwrap().inventory_ops[0] = Some("Drink".into()),
            9 => {
                o.remove("jangerberries");
            }
            _ => {
                o.insert("jangerberries".into(), 248);
            }
        }
        let mut out = BTreeMap::new();
        out.insert(1999, f.clone());
        let before = format!("{out:?}");
        assert!(
            jangerberries(&o, &stat, &q, &mut out).is_err(),
            "itemcase{mode}"
        );
        assert_eq!(format!("{out:?}"), before);
        n += 1;
    }
    for (id, symbol) in [
        (0u32, "attack"),
        (1, "defence"),
        (2, "strength"),
        (5, "prayer"),
    ] {
        for mode in 0..7 {
            let mut q = copy(&d);
            let mut s = stat.clone();
            match mode {
                0 => {
                    q.stats.remove(&id);
                }
                1 => q.stats.get_mut(&id).unwrap().name = "Wrong".into(),
                2 => q.stats.get_mut(&id).unwrap().unreleased = true,
                3 => q.stats.get_mut(&id).unwrap().min_level = 0,
                4 => q.stats.get_mut(&id).unwrap().max_level = 100,
                5 => {
                    s.remove(symbol);
                }
                _ => {
                    s.insert(symbol.into(), 23);
                }
            }
            let mut out = BTreeMap::new();
            assert!(
                jangerberries(&obj, &s, &q, &mut out).is_err(),
                "stat{id} mode{mode}"
            );
            assert!(out.is_empty());
            n += 1;
        }
    }
    assert_eq!(n, 40);
    println!("PG_JANGER_BINDINGS actual247memberEat fourreleasedstats HP2 Attack2 Strength1 PrayerHeal1 DefenceSubtract1 and40atomicbindingrefusals PASS");
}
