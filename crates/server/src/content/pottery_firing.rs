//! Five ordinary firing outcomes; cracks consume raw with no output or XP.
use super::{Definitions, Symbols};
use gielinor_world::facility_crafting::{Config, Failure, Recipe};
pub fn pottery_firing(
    objects: &Symbols,
    locations: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Config, String> {
    if stats.get("crafting") != Some(&12)
        || locations.get("potteryoven") != Some(&14888)
        || sequences.get("potteryoven_quick") != Some(&1317)
    {
        return Err("Missing/aliased pottery firing bindings".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    let loc = defs
        .locations
        .get(&14888)
        .ok_or("Missing ordinary pottery oven")?;
    if loc.name != "Pottery Oven"
        || loc.ops[0].as_deref() != Some("Fire")
        || loc.collision.width != 1
        || loc.collision.length != 2
        || !loc.multi.is_empty()
        || loc.multi_default != -1
        || loc.multi_varbit != -1
        || loc.multi_varp != -1
    {
        return Err("Invalid ordinary pottery oven".into());
    }
    let check = |symbol: &str, id: i32, name: &str, member: bool| -> Result<(), String> {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased pottery firing item {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing pottery firing item")?;
        if d.name != name
            || d.members != member
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid pottery firing item {id}"));
        }
        Ok(())
    };
    let mut recipes = Vec::new();
    for (raw, raw_symbol, raw_name, output, out_symbol, name, xp_tenths, members) in [
        (
            1787,
            "pot_unfired",
            "Unfired pot",
            1931,
            "pot_empty",
            "Pot",
            63,
            false,
        ),
        (
            28193,
            "cup_unfired",
            "Unfired cup",
            1980,
            "cup_empty",
            "Empty cup",
            85,
            true,
        ),
        (
            1789,
            "piedish_unfired",
            "Unfired pie dish",
            2313,
            "piedish",
            "Pie dish",
            100,
            false,
        ),
        (
            1791,
            "bowl_unfired",
            "Unfired bowl",
            1923,
            "bowl_empty",
            "Bowl",
            150,
            false,
        ),
        (
            5352,
            "plantpot_unfired",
            "Unfired plant pot",
            5350,
            "plantpot_empty",
            "Empty plant pot",
            175,
            true,
        ),
    ] {
        check(raw_symbol, raw, raw_name, members)?;
        check(out_symbol, output, name, members)?;
        recipes.push(Recipe {
            input: raw,
            input_count: 1,
            output,
            output_count: 1,
            level: 1,
            xp_tenths,
            members,
            first_ticks: 6,
            cycle_ticks: 7,
            failure: Some(Failure {
                low: 180,
                high: 800,
                output: None,
                xp_tenths: 0,
                message: "The clay cracks in the oven and is ruined.",
            }),
            label: name,
        });
    }
    Ok(Config {
        locations: [(14888, Some(1))].into(),
        recipes: std::sync::Arc::new(recipes),
        action: "fire",
        empty_message: "You don't have any suitable unfired pottery.",
        animation: 1317,
        location_animation: None,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        facility_crafting,
        inventory::{Inventory, Item, StackType},
        npc::RandomSource,
        stats::Stats,
    };
    struct Draw {
        value: u32,
        calls: usize,
    }
    impl RandomSource for Draw {
        fn below(&mut self, n: u32) -> u32 {
            assert_eq!(n, 256);
            self.calls += 1;
            self.value
        }
    }
    #[test]
    fn pottery_firing_all_at_level_one_stage_success_or_zero_xp_destruction_before_roll() {
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        defs.stats.insert(
            12,
            gielinor_cache::definitions::StatType::decode(b"\x01Crafting\0\0").unwrap(),
        );
        defs.locations.insert(
            14888,
            gielinor_cache::locations::LocationType::decode(
                b"\x02Pottery Oven\0\x0f\x02\x1eFire\0\0",
            )
            .unwrap(),
        );
        let mut obj = Symbols::new();
        for (id, symbol, name, member) in [
            (1787, "pot_unfired", "Unfired pot", false),
            (1931, "pot_empty", "Pot", false),
            (28193, "cup_unfired", "Unfired cup", true),
            (1980, "cup_empty", "Empty cup", true),
            (1789, "piedish_unfired", "Unfired pie dish", false),
            (2313, "piedish", "Pie dish", false),
            (1791, "bowl_unfired", "Unfired bowl", false),
            (1923, "bowl_empty", "Bowl", false),
            (5352, "plantpot_unfired", "Unfired plant pot", true),
            (5350, "plantpot_empty", "Empty plant pot", true),
        ] {
            obj.insert(symbol.into(), id);
            let mut b = vec![2];
            b.extend_from_slice(name.as_bytes());
            b.push(0);
            if member {
                b.push(16);
            }
            b.push(0);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&b).unwrap(),
            );
        }
        let loc = [("potteryoven".into(), 14888)].into();
        let seq = [("potteryoven_quick".into(), 1317)].into();
        let stat = [("crafting".into(), 12)].into();
        let config = pottery_firing(&obj, &loc, &seq, &stat, &defs).unwrap();
        assert_eq!(config.recipes.len(), 5);
        let cat = defs.inventory_catalog();
        for (recipe, fine) in config.recipes.iter().copied().zip([78, 106, 125, 187, 218]) {
            let mut inv = Inventory::new(1, StackType::Normal, false);
            inv.restore_slot(
                0,
                Some(Item {
                    id: recipe.input,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
            let mut stats = Stats::default();
            for (level, roll, success) in [
                (1, 180, true),
                (1, 181, false),
                (12, 250, true),
                (12, 251, false),
                (13, 255, true),
            ] {
                stats.set_current(12, level);
                let mut rng = Draw {
                    value: roll,
                    calls: 0,
                };
                let ((next, next_stats, advanced), actual) = facility_crafting::resolve_group(
                    recipe,
                    &[0],
                    &inv,
                    &cat,
                    &stats,
                    125,
                    true,
                    &mut rng,
                )
                .unwrap();
                assert_eq!(actual, success);
                assert_eq!(rng.calls, 1);
                if success {
                    assert_eq!(next.get(0).unwrap().id, recipe.output);
                    assert_eq!(next_stats.fine_xp(12), fine);
                    assert!(advanced.is_some());
                } else {
                    assert!(next.get(0).is_none());
                    assert_eq!(next_stats, stats);
                    assert!(advanced.is_none());
                }
            }
            for bad_failure in [false, true] {
                let mut r = recipe;
                let mut bad = defs.inventory_catalog();
                if bad_failure {
                    r.failure.as_mut().unwrap().output = Some((999, 1));
                    bad.dummy.insert(999);
                } else {
                    bad.dummy.insert(recipe.output);
                }
                let before = (inv.clone(), stats.clone());
                let mut rng = Draw { value: 0, calls: 0 };
                assert!(facility_crafting::resolve_group(
                    r,
                    &[0],
                    &inv,
                    &bad,
                    &stats,
                    125,
                    true,
                    &mut rng
                )
                .is_err());
                assert_eq!(rng.calls, 0);
                assert_eq!((inv.clone(), stats.clone()), before);
            }
        }
        defs.items.get_mut(&5350).unwrap().cert.template = 799;
        assert!(pottery_firing(&obj, &loc, &seq, &stat, &defs).is_err());
    }
}
