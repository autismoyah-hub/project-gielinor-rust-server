//! Actual cache-backed factory calls; not native or startup acceptance.
use gielinor_cache::{
    definitions::{ItemType, StatType},
    game::{Definitions, GameCache},
    symbols,
};
use gielinor_server::content::wine_drink;
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
fn actual_wine_bindings_and_atomic_malformed_registration_refusals() {
    let dir = std::env::var("PG_SYMBOLS").unwrap();
    let names = |kind| symbols::load(&[Path::new(&dir)], kind, None).unwrap();
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
    for id in [1993, 1935] {
        d.items.insert(id, ItemType::decode(&items[&id]).unwrap());
    }
    for id in [0, 3] {
        d.stats.insert(id, StatType::decode(&stats[&id]).unwrap());
    }
    let mut food = BTreeMap::new();
    let (id, effect) = wine_drink(&obj, &stat, &d, &mut food).unwrap();
    assert_eq!((id, effect.skill, effect.amount), (1993, 0, 2));
    let f = &food[&id];
    assert_eq!((f.heal, f.delay, f.replacement), (11, 3, Some(1935)));
    assert_eq!(f.message, "You drink the wine.");
    let original = food.clone();
    assert!(wine_drink(&obj, &stat, &d, &mut food).is_err());
    assert_eq!(format!("{food:?}"), format!("{original:?}"));
    let mut n = 1;
    for id in [1993u32, 1935] {
        for mode in 0..11 {
            let mut q = copy(&d);
            let mut o = obj.clone();
            match mode {
                0 => {
                    q.items.remove(&id);
                }
                1 => q.items.get_mut(&id).unwrap().name = "Wrong".into(),
                2 => q.items.get_mut(&id).unwrap().members = true,
                3 => q.items.get_mut(&id).unwrap().stackable = true,
                4 => q.items.get_mut(&id).unwrap().dummy = 1,
                5 => q.items.get_mut(&id).unwrap().cert.template = 799,
                6 => q.items.get_mut(&id).unwrap().placeholder.template = 14401,
                7 => q.items.get_mut(&id).unwrap().transform.template = 1,
                8 => q.items.get_mut(&id).unwrap().inventory_ops[0] = Some("Eat".into()),
                9 => {
                    o.retain(|_, v| *v != id as i32);
                }
                _ => {
                    for v in o.values_mut() {
                        if *v == id as i32 {
                            *v = 999999;
                        }
                    }
                }
            }
            let mut f = BTreeMap::new();
            f.insert(1999, original[&1993].clone());
            let before = format!("{f:?}");
            assert!(
                wine_drink(&o, &stat, &q, &mut f).is_err(),
                "item{id} mode{mode}"
            );
            assert_eq!(format!("{f:?}"), before);
            n += 1;
        }
    }
    for id in [0u32, 3] {
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
                5 => s.retain(|_, v| *v != id as i32),
                _ => {
                    for v in s.values_mut() {
                        if *v == id as i32 {
                            *v = 23;
                        }
                    }
                }
            }
            let mut f = BTreeMap::new();
            assert!(
                wine_drink(&obj, &s, &q, &mut f).is_err(),
                "stat{id} mode{mode}"
            );
            assert!(f.is_empty());
            n += 1;
        }
    }
    assert_eq!(n, 37);
    println!("PG_WINE_BINDINGS actual2items2stats factory11HP jug1935 currentAttackSubtract2 and37atomicregistrationrefusals PASS");
}
