//! Actual239.1 target definitions only; this does not claim startup/factory execution.
use gielinor_cache::{
    definitions::{ItemType, Param, StatType},
    game::GameCache,
    npcs::NpcType,
    symbols,
};
use std::path::Path;
#[test]
fn seven_items_two_sellers_and_released_skill_mask_from_actual_cache() {
    let names = std::env::var("PG_SYMBOLS").unwrap();
    let names = |kind| symbols::load(&[Path::new(&names)], kind, None).unwrap();
    let objects = names("obj");
    let npcs = names("npc");
    let stats = names("stat");
    let params = names("param");
    assert_eq!(params.get("statreq1_skill"), Some(&434));
    assert_eq!(params.get("statreq1_level"), Some(&436));
    let path = std::env::var("PG_GAME_CACHE").unwrap();
    let mut c = GameCache::open(Path::new(&path), 0xff_ffff).unwrap();
    let items = c.files(2, 10, None).unwrap().unwrap();
    let npc = c.files(2, 9, None).unwrap().unwrap();
    let stat = c.files(2, 200, None).unwrap().unwrap();
    for (sym, id, name, skill) in [
        ("coins", 995, "Coins", None),
        ("skillcape_crafting", 9780, "Crafting cape", Some(12)),
        (
            "skillcape_crafting_trimmed",
            9781,
            "Crafting cape(t)",
            Some(12),
        ),
        ("skillcape_crafting_hood", 9782, "Crafting hood", Some(12)),
        ("skillcape_cooking", 9801, "Cooking cape", Some(7)),
        (
            "skillcape_cooking_trimmed",
            9802,
            "Cooking cape(t)",
            Some(7),
        ),
        ("skillcape_cooking_hood", 9803, "Cooking hood", Some(7)),
    ] {
        assert_eq!(objects.get(sym), Some(&id));
        let d = ItemType::decode(&items[&(id as u32)]).unwrap();
        assert_eq!(d.name, name);
        assert_eq!(d.members, skill.is_some());
        assert_eq!(d.stackable, skill.is_none());
        assert_ne!(d.dummy, 1);
        assert_eq!(
            (
                d.cert.template,
                d.placeholder.template,
                d.transform.template
            ),
            (0, 0, 0)
        );
        if let Some(s) = skill {
            assert_eq!(
                d.wear,
                if [9782, 9803].contains(&id) {
                    [0, 8, -1]
                } else {
                    [1, -1, -1]
                }
            );
            assert_eq!(d.inventory_ops[1].as_deref(), Some("Wear"));
            assert_eq!(d.params.get(&434), Some(&Param::Integer(s)));
            assert_eq!(d.params.get(&436), Some(&Param::Integer(99)));
        }
        println!(
            "PG_VENDOR_CACHE_ITEM id{id} member={} stackable={} wear={:?} ordinary",
            d.members, d.stackable, d.wear
        );
    }
    for (sym, id, name) in [
        ("master_crafter", 5810, "Master Crafter"),
        ("head_chef", 2658, "Head chef"),
    ] {
        assert_eq!(npcs.get(sym), Some(&id));
        let d = NpcType::decode(&npc[&(id as u32)]).unwrap();
        assert_eq!(d.name, name);
        assert_eq!(d.size, 1);
        assert_eq!(d.ops[0].as_deref(), Some("Talk-to"));
        assert!(
            d.multi.is_empty()
                && d.multi_default.is_none()
                && d.multi_varbit.is_none()
                && d.multi_varp.is_none()
        );
        println!("PG_VENDOR_CACHE_NPC id{id} name={name:?} size1 Talk-to static");
    }
    let mut mask = 0u32;
    for (&id, raw) in &stat {
        let d = StatType::decode(raw).unwrap();
        if id < 24 && !d.unreleased && d.max_level == 99 {
            mask |= 1 << id;
        }
        if id == 7 || id == 12 {
            assert_eq!(d.name, if id == 7 { "Cooking" } else { "Crafting" });
            assert_eq!(d.min_level, 1);
            assert_eq!(d.max_level, 99);
            assert!(!d.unreleased);
        }
    }
    assert_eq!(stats.get("cooking"), Some(&7));
    assert_eq!(stats.get("crafting"), Some(&12));
    assert_eq!(mask, 0x007fffff);
    println!("PG_VENDOR_CACHE_FACTS7items2NPC2skills2params actualreleasedmask={mask:08x};facts only,no serverfactory/startup/native acceptance PASS");
}
