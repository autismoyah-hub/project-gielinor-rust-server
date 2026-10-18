//! Manual member-fish profiles over the host's existing cooking transaction.
use super::{Definitions, Symbols};
use gielinor_world::cooking::{Config, Recipe};

pub fn member_fish_cooking(
    objects: &Symbols,
    locations: &Symbols,
    defs: &Definitions,
    config: &mut Config,
) -> Result<(), String> {
    let range = u32::try_from(*locations.get("range").ok_or("Missing member-fish range")?)
        .map_err(|_| "Invalid member-fish range")?;
    let mut fire = config.recipes.clone();
    let mut ranged = config
        .surfaces
        .get(&range)
        .ok_or("Configure ordinary range before member fish")?
        .recipes
        .clone();
    let mut members = config.member_inputs.clone();
    for (
        (raw_symbol, cooked_symbol, burnt_symbol),
        (raw_name, cooked_name, burnt_name),
        level,
        fine_xp,
        timing,
        fire_rule,
        range_rule,
        burnt_id,
        burnt_members,
    ) in [
        (
            ("raw_mackerel", "mackerel", "burntfish3"),
            ("Raw mackerel", "Mackerel", "Burnt fish"),
            10,
            600,
            (4, 4),
            (Some(45), 98, 452),
            (Some(45), 98, 452),
            357,
            false,
        ),
        (
            ("raw_cod", "cod", "burntfish2"),
            ("Raw cod", "Cod", "Burnt fish"),
            18,
            750,
            (4, 4),
            (Some(51), 83, 422),
            (Some(49), 88, 432),
            343,
            false,
        ),
        (
            ("raw_bass", "bass", "burntfish4"),
            ("Raw bass", "Bass", "Burnt fish"),
            43,
            1300,
            (4, 4),
            (Some(80), 33, 312),
            (Some(80), 33, 312),
            367,
            false,
        ),
        (
            ("raw_monkfish", "monkfish", "burnt_monkfish"),
            ("Raw monkfish", "Monkfish", "Burnt monkfish"),
            62,
            1500,
            (4, 1),
            (Some(92), 11, 275),
            (Some(90), 13, 280),
            7948,
            true,
        ),
        (
            ("raw_shark", "shark", "burnt_shark"),
            ("Raw shark", "Shark", "Burnt shark"),
            80,
            2100,
            (4, 4),
            (None, 1, 202),
            (None, 1, 232),
            387,
            true,
        ),
        (
            ("raw_seaturtle", "seaturtle", "burnt_seaturtle"),
            ("Raw sea turtle", "Sea turtle", "Burnt sea turtle"),
            82,
            2113,
            (4, 1),
            (None, 1, 202),
            (None, 1, 222),
            399,
            true,
        ),
        (
            ("raw_mantaray", "mantaray", "burnt_mantaray"),
            ("Raw manta ray", "Manta ray", "Burnt manta ray"),
            91,
            2163,
            (4, 1),
            (None, 1, 202),
            (None, 1, 222),
            393,
            true,
        ),
        (
            ("raw_dark_crab", "dark_crab", "burnt_dark_crab"),
            ("Raw dark crab", "Dark crab", "Burnt dark crab"),
            90,
            2150,
            (4, 4),
            (None, 10, 222),
            (None, 10, 222),
            11938,
            true,
        ),
        (
            ("raw_anglerfish", "anglerfish", "burnt_anglerfish"),
            ("Raw anglerfish", "Anglerfish", "Burnt anglerfish"),
            84,
            2300,
            (4, 4),
            (None, 1, 200),
            (None, 1, 220),
            13443,
            true,
        ),
        (
            ("raw_cave_eel", "cave_eel", "burnt_cave_eel"),
            ("Raw cave eel", "Cave eel", "Burnt cave eel"),
            38,
            1150,
            (4, 1),
            (Some(74), 38, 332),
            (Some(74), 38, 332),
            5002,
            true,
        ),
        (
            ("raw_lava_eel", "lava_eel", "burnt_eel"),
            ("Raw lava eel", "Lava eel", "Burnt eel"),
            // The donor's burnt-eel metadata is required by the shared recipe
            // shape but unreachable: cutoff equals the minimum Cooking level.
            53,
            300,
            (4, 1),
            (Some(53), 50, 256),
            (Some(53), 50, 256),
            3383,
            true,
        ),
    ] {
        let resolve = |n: &str| {
            objects
                .get(n)
                .copied()
                .ok_or_else(|| format!("Missing member-fish symbol {n}"))
        };
        let raw = resolve(raw_symbol)?;
        let cooked = resolve(cooked_symbol)?;
        let burnt = resolve(burnt_symbol)?;
        if raw == cooked
            || raw == burnt
            || cooked == burnt
            || burnt != burnt_id
            || fire.contains_key(&raw)
            || ranged.contains_key(&raw)
            || fire
                .values()
                .chain(ranged.values())
                .any(|r| r.cooked == raw || r.burnt == raw || r.raw == cooked || r.raw == burnt)
        {
            return Err("Aliased or unattested member-fish variant".into());
        }
        for (id, name, member) in [
            (raw, raw_name, true),
            (cooked, cooked_name, true),
            (burnt, burnt_name, burnt_members),
        ] {
            let d = u32::try_from(id)
                .ok()
                .and_then(|v| defs.items.get(&v))
                .ok_or("Missing member-fish definition")?;
            if d.name != name
                || d.members != member
                || d.stackable
                || d.dummy == 1
                || d.cert.template != 0
                || d.placeholder.template != 0
                || d.transform.template != 0
            {
                return Err("Unattested ordinary member-fish item".into());
            }
        }
        if defs.items[&(cooked as u32)].inventory_ops[0].as_deref() != Some("Eat") {
            return Err("Cooked member fish lacks Eat".into());
        }
        let make = |rule: (Option<u8>, i32, i32), ticks| Recipe {
            raw,
            cooked,
            burnt,
            level,
            stop_burn: rule.0,
            low: rule.1,
            high: rule.2,
            xp: (fine_xp / 10) as u16,
            fine_xp_override: (fine_xp % 10 != 0).then_some(fine_xp),
            ticks,
        };
        fire.insert(raw, make(fire_rule, timing.0));
        ranged.insert(raw, make(range_rule, timing.1));
        members.insert(raw);
    }
    config.recipes = fire;
    config.surfaces.get_mut(&range).unwrap().recipes = ranged;
    config.member_inputs = members;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        cooking::Surface,
        inventory::{Catalog, Inventory, Item, StackType},
        npc::RandomSource,
        stats::Stats,
    };
    use std::collections::BTreeMap;
    struct Draw(u32);
    impl RandomSource for Draw {
        fn below(&mut self, n: u32) -> u32 {
            assert_eq!(n, 256);
            self.0
        }
    }
    #[test]
    fn member_fish_exact_variants_method_rules_fractional_awards_and_atomic_publication() {
        let mut objects = Symbols::new();
        let locations = Symbols::from([("range".into(), 26181)]);
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (symbol, id, name, member, eat) in [
            ("raw_mackerel", 353, "Raw mackerel", true, false),
            ("mackerel", 355, "Mackerel", true, true),
            ("burntfish3", 357, "Burnt fish", false, false),
            ("raw_cod", 341, "Raw cod", true, false),
            ("cod", 339, "Cod", true, true),
            ("burntfish2", 343, "Burnt fish", false, false),
            ("raw_bass", 363, "Raw bass", true, false),
            ("bass", 365, "Bass", true, true),
            ("burntfish4", 367, "Burnt fish", false, false),
            ("raw_monkfish", 7944, "Raw monkfish", true, false),
            ("monkfish", 7946, "Monkfish", true, true),
            ("burnt_monkfish", 7948, "Burnt monkfish", true, false),
            ("raw_shark", 383, "Raw shark", true, false),
            ("shark", 385, "Shark", true, true),
            ("burnt_shark", 387, "Burnt shark", true, false),
            ("raw_seaturtle", 395, "Raw sea turtle", true, false),
            ("seaturtle", 397, "Sea turtle", true, true),
            ("burnt_seaturtle", 399, "Burnt sea turtle", true, false),
            ("raw_mantaray", 389, "Raw manta ray", true, false),
            ("mantaray", 391, "Manta ray", true, true),
            ("burnt_mantaray", 393, "Burnt manta ray", true, false),
            ("raw_dark_crab", 11934, "Raw dark crab", true, false),
            ("dark_crab", 11936, "Dark crab", true, true),
            ("burnt_dark_crab", 11938, "Burnt dark crab", true, false),
            ("raw_anglerfish", 13439, "Raw anglerfish", true, false),
            ("anglerfish", 13441, "Anglerfish", true, true),
            ("burnt_anglerfish", 13443, "Burnt anglerfish", true, false),
            ("raw_cave_eel", 5001, "Raw cave eel", true, false),
            ("cave_eel", 5003, "Cave eel", true, true),
            ("burnt_cave_eel", 5002, "Burnt cave eel", true, false),
            ("raw_lava_eel", 2148, "Raw lava eel", true, false),
            ("lava_eel", 2149, "Lava eel", true, true),
            ("burnt_eel", 3383, "Burnt eel", true, false),
        ] {
            objects.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            d.name = name.into();
            d.members = member;
            if eat {
                d.inventory_ops[0] = Some("Eat".into());
            }
            defs.items.insert(id as u32, d);
        }
        let empty = Config {
            fire: 26185,
            animation: 897,
            skill: 7,
            max_level: 99,
            tutorial: None,
            gauntlets: None, no_burn_cape: None, member_inputs: Default::default(),
            recipes: BTreeMap::new(),
            surfaces: [(
                26181,
                Surface {
                    animation: 896,
                    recipes: BTreeMap::new(),
                },
            )]
            .into(),
        };
        let mut config = empty.clone();
        member_fish_cooking(&objects, &locations, &defs, &mut config).unwrap();
        assert_eq!(config.recipes.len(), 11);
        assert_eq!(config.surfaces[&26181].recipes.len(), 11);
        assert_eq!(config.member_inputs.len(), 11);
        for (source, recipes) in [
            (26185, &config.recipes),
            (26181, &config.surfaces[&26181].recipes),
        ] {
            for (raw, cooked, burnt, fine125) in [
                (353, 355, 357, 750),
                (341, 339, 343, 937),
                (363, 365, 367, 1625),
                (7944, 7946, 7948, 1875),
                (383, 385, 387, 2625),
                (395, 397, 399, 2641),
                (389, 391, 393, 2703),
                (11934, 11936, 11938, 2687),
                (13439, 13441, 13443, 2875),
                (5001, 5003, 5002, 1437),
                (2148, 2149, 3383, 375),
            ] {
                let recipe = &recipes[&raw];
                assert_eq!((recipe.cooked, recipe.burnt), (cooked, burnt));
                assert!(config.member_inputs.contains(&raw));
                assert_eq!(
                    recipe.ticks,
                    if source == 26181 && [7944, 395, 389, 5001, 2148].contains(&raw) {
                        1
                    } else {
                        4
                    }
                );
                for burned in [false, true] {
                    if raw == 2148 && burned {
                        continue;
                    }
                    let mut inv = Inventory::new(2, StackType::Normal, false);
                    for slot in 0..2 {
                        inv.restore_slot(
                            slot,
                            Some(Item {
                                id: raw,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    }
                    let mut stats = Stats::default();
                    stats.set_current(7, recipe.level as i8);
                    let outcome = recipe
                        .complete(
                            1,
                            &mut inv,
                            &Catalog::default(),
                            &mut stats,
                            7,
                            99,
                            125,
                            &mut Draw(if burned { 255 } else { 0 }),
                        )
                        .unwrap();
                    assert_eq!(outcome.burned, burned);
                    assert_eq!(inv.get(0).unwrap().id, raw);
                    assert_eq!(inv.get(1).unwrap().id, if burned { burnt } else { cooked });
                    assert_eq!(stats.fine_xp(7), if burned { 0 } else { fine125 });
                }
                if recipe.stop_burn.is_none() {
                    let mut inv = Inventory::new(1, StackType::Normal, false);
                    inv.restore_slot(
                        0,
                        Some(Item {
                            id: raw,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                    let mut stats = Stats::default();
                    stats.set_current(7, 99);
                    assert!(
                        recipe
                            .complete(
                                0,
                                &mut inv,
                                &Catalog::default(),
                                &mut stats,
                                7,
                                99,
                                100,
                                &mut Draw(255)
                            )
                            .unwrap()
                            .burned
                    );
                    assert_eq!(stats.fine_xp(7), 0);
                }
            }
        }
        // Lava eels cannot burn: even at the minimum level there is no RNG
        // call. A below-level request retains both inventory and fine XP.
        struct NeverDraw;
        impl RandomSource for NeverDraw {
            fn below(&mut self, _: u32) -> u32 {
                panic!("Guaranteed lava eel consumed RNG")
            }
        }
        for source in [26185, 26181] {
            let recipe = config.recipe(source, 2148).unwrap();
            assert_eq!(recipe.stop_burn, Some(recipe.level));
            let mut inv = Inventory::new(1, StackType::Normal, false);
            inv.restore_slot(
                0,
                Some(Item {
                    id: 2148,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
            let mut stats = Stats::default();
            stats.set_current(7, 52);
            let before = (inv.clone(), stats.clone());
            assert!(matches!(
                recipe.complete(
                    0,
                    &mut inv,
                    &Catalog::default(),
                    &mut stats,
                    7,
                    99,
                    100,
                    &mut NeverDraw
                ),
                Err(gielinor_world::cooking::Error::Level)
            ));
            assert_eq!((inv.clone(), stats.clone()), before);
            stats.set_current(7, 53);
            assert!(
                !recipe
                    .complete(
                        0,
                        &mut inv,
                        &Catalog::default(),
                        &mut stats,
                        7,
                        99,
                        100,
                        &mut NeverDraw
                    )
                    .unwrap()
                    .burned
            );
            assert_eq!(inv.get(0).unwrap().id, 2149);
            assert_eq!(stats.fine_xp(7), 300);
            let cave = config.recipe(source, 5001).unwrap();
            assert_eq!((cave.stop_burn, cave.low, cave.high), (Some(74), 38, 332));
        }
        let cod_fire = config.recipe(26185, 341).unwrap();
        let cod_range = config.recipe(26181, 341).unwrap();
        assert_eq!(
            (cod_fire.stop_burn, cod_fire.low, cod_fire.high),
            (Some(51), 83, 422)
        );
        assert_eq!(
            (cod_range.stop_burn, cod_range.low, cod_range.high),
            (Some(49), 88, 432)
        );
        for (raw, rate, expected) in [
            (395, 100, 2113),
            (395, 500, 10565),
            (389, 100, 2163),
            (389, 500, 10815),
        ] {
            let recipe = config.recipe(26181, raw).unwrap();
            let mut inv = Inventory::new(1, StackType::Normal, false);
            inv.restore_slot(
                0,
                Some(Item {
                    id: raw,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
            let mut stats = Stats::default();
            stats.set_current(7, recipe.level as i8);
            recipe
                .complete(
                    0,
                    &mut inv,
                    &Catalog::default(),
                    &mut stats,
                    7,
                    99,
                    rate,
                    &mut Draw(0),
                )
                .unwrap();
            assert_eq!(stats.fine_xp(7), expected);
        }
        let mut failed = empty.clone();
        defs.items.get_mut(&13443).unwrap().dummy = 1;
        assert!(member_fish_cooking(&objects, &locations, &defs, &mut failed).is_err());
        assert!(failed.recipes.is_empty());
        assert!(failed.surfaces[&26181].recipes.is_empty());
        assert!(failed.member_inputs.is_empty());
        defs.items.get_mut(&13443).unwrap().dummy = -1;
        objects.insert("burntfish3".into(), 343);
        assert!(member_fish_cooking(&objects, &locations, &defs, &mut failed).is_err());
        assert!(failed.recipes.is_empty());
    }
}
