//! Classic grouped-material loom recipes, with explicit Wiki Make-all timings.
use super::{Definitions, Symbols};
use gielinor_world::facility_crafting::{Config, Recipe};
pub fn weaving_loom(
    objects: &Symbols,
    locations: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Config, String> {
    if stats.get("crafting") != Some(&12)
        || locations.get("loom") != Some(&8717)
        || sequences.get("farming_useloom") != Some(&2270)
    {
        return Err("Missing/aliased loom bindings".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    let loc = defs.locations.get(&8717).ok_or("Missing ordinary loom")?;
    if loc.name != "Loom"
        || loc.ops[1].as_deref() != Some("Weave")
        || loc.collision.width != 1
        || loc.collision.length != 1
        || !loc.multi.is_empty()
        || loc.multi_default != -1
        || loc.multi_varbit != -1
        || loc.multi_varp != -1
    {
        return Err("Invalid ordinary loom".into());
    }
    for (symbol, id, name, member) in [
        ("ball_of_wool", 1759, "Ball of wool", false),
        ("regicide_cloth", 3224, "Strip of cloth", true),
        ("jute_fibre", 5931, "Jute fibre", true),
        ("sack_empty", 5418, "Empty sack", true),
        ("fossil_drift_net", 21652, "Drift net", true),
        ("willow_branch", 5933, "Willow branch", true),
        ("basket_empty", 5376, "Basket", true),
    ] {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased weaving item {symbol}"));
        }
        let d = defs.items.get(&(id as u32)).ok_or("Missing weaving item")?;
        if d.name != name
            || d.members != member
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid weaving item {id}"));
        }
    }
    Ok(Config {
        locations: [(8717, Some(2))].into(),
        recipes: std::sync::Arc::new(vec![
            Recipe {
                input: 1759,
                input_count: 4,
                output_count: 1,
                output: 3224,
                level: 10,
                xp_tenths: 120,
                members: true,
                first_ticks: 4,
                cycle_ticks: 3,
                failure: None,
                label: "strip of cloth",
            },
            Recipe {
                input: 5931,
                input_count: 4,
                output_count: 1,
                output: 5418,
                level: 21,
                xp_tenths: 380,
                members: true,
                first_ticks: 4,
                cycle_ticks: 3,
                failure: None,
                label: "empty sack",
            },
            Recipe {
                input: 5931,
                input_count: 2,
                output_count: 1,
                output: 21652,
                level: 26,
                xp_tenths: 550,
                members: true,
                first_ticks: 3,
                cycle_ticks: 3,
                failure: None,
                label: "drift net",
            },
            Recipe {
                input: 5933,
                input_count: 6,
                output_count: 1,
                output: 5376,
                level: 36,
                xp_tenths: 560,
                members: true,
                first_ticks: 4,
                cycle_ticks: 3,
                failure: None,
                label: "basket",
            },
        ]),
        action: "weave",
        empty_message: "You don't have enough suitable materials to weave anything at this loom.",
        animation: 2270,
        location_animation: None,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        facility_crafting,
        inventory::{Inventory, Item, StackType},
        stats::Stats,
    };
    #[test]
    fn weaving_loom_four_material_groups_have_atomic_per_product_rewards_and_no_partial_credit() {
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
            8717,
            gielinor_cache::locations::LocationType::decode(b"\x02Loom\0\x1fWeave\0\0").unwrap(),
        );
        let mut obj = Symbols::new();
        for (id, symbol, name, member) in [
            (1759, "ball_of_wool", "Ball of wool", false),
            (3224, "regicide_cloth", "Strip of cloth", true),
            (5931, "jute_fibre", "Jute fibre", true),
            (5418, "sack_empty", "Empty sack", true),
            (21652, "fossil_drift_net", "Drift net", true),
            (5933, "willow_branch", "Willow branch", true),
            (5376, "basket_empty", "Basket", true),
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
        let loc = [("loom".into(), 8717)].into();
        let seq = [("farming_useloom".into(), 2270)].into();
        let stat = [("crafting".into(), 12)].into();
        let config = weaving_loom(&obj, &loc, &seq, &stat, &defs).unwrap();
        assert_eq!(config.recipes.len(), 4);
        let cat = defs.inventory_catalog();
        for (recipe, fine) in config.recipes.iter().copied().zip([150, 475, 687, 700]) {
            let mut inv = Inventory::new(28, StackType::Normal, false);
            for slot in 0..28 {
                inv.restore_slot(
                    slot,
                    Some(Item {
                        id: 1511,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
            let slots: Vec<_> = (0..usize::from(recipe.input_count)).collect();
            for &slot in &slots {
                inv.restore_slot(
                    slot,
                    Some(Item {
                        id: recipe.input,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
            let mut stats = Stats::default();
            stats.set_base(12, recipe.level as i8);
            stats.set_current(12, recipe.level as i8);
            let (next, next_stats, _) =
                facility_crafting::prepare_group(recipe, &slots, &inv, &cat, &stats, 125, true)
                    .unwrap();
            assert_eq!(next_stats.fine_xp(12), fine);
            assert_eq!(next.free_space(), usize::from(recipe.input_count) - 1);
            assert_eq!(
                next.slots()
                    .iter()
                    .flatten()
                    .filter(|v| v.id == recipe.output)
                    .count(),
                1
            );
            let before = (inv.clone(), stats.clone());
            let mut bad = defs.inventory_catalog();
            bad.dummy.insert(recipe.output);
            assert!(facility_crafting::prepare_group(
                recipe, &slots, &inv, &bad, &stats, 125, true
            )
            .is_err());
            assert!(facility_crafting::prepare_group(
                recipe,
                &slots[..slots.len() - 1],
                &inv,
                &cat,
                &stats,
                125,
                true
            )
            .is_err());
            let mut duplicate = slots.clone();
            duplicate[1] = duplicate[0];
            assert!(facility_crafting::prepare_group(
                recipe, &duplicate, &inv, &cat, &stats, 125, true
            )
            .is_err());
            assert_eq!((inv, stats), before);
        }
        defs.items.get_mut(&5376).unwrap().cert.template = 799;
        assert!(weaving_loom(&obj, &loc, &seq, &stat, &defs).is_err());
    }
}
