//! Five ordinary pottery shaping recipes; quest-gated pot lids remain separate.
use super::{Definitions, Symbols};
use gielinor_world::facility_crafting::{Config, Recipe};
pub fn pottery_shaping(
    objects: &Symbols,
    locations: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Config, String> {
    if stats.get("crafting") != Some(&12)
        || locations.get("potterywheel") != Some(&14887)
        || sequences.get("human_potterywheel") != Some(&883)
    {
        return Err("Missing/aliased pottery shaping bindings".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    let loc = defs
        .locations
        .get(&14887)
        .ok_or("Missing ordinary potter wheel")?;
    if loc.name != "Potter's Wheel"
        || loc.ops[0].as_deref() != Some("Use")
        || loc.collision.width != 1
        || loc.collision.length != 1
        || !loc.multi.is_empty()
        || loc.multi_default != -1
        || loc.multi_varbit != -1
        || loc.multi_varp != -1
    {
        return Err("Invalid ordinary potter wheel".into());
    }
    let check = |symbol: &str, id: i32, name: &str, member: bool| -> Result<(), String> {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased pottery shaping item {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing pottery shaping item")?;
        if d.name != name
            || d.members != member
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid pottery shaping item {id}"));
        }
        Ok(())
    };
    check("softclay", 1761, "Soft clay", false)?;
    let mut recipes = Vec::new();
    for (symbol, id, name, level, xp_tenths, output_count, members) in [
        ("pot_unfired", 1787, "Unfired pot", 1, 63, 1, false),
        ("cup_unfired", 28193, "Unfired cup", 3, 85, 4, true),
        (
            "piedish_unfired",
            1789,
            "Unfired pie dish",
            7,
            150,
            1,
            false,
        ),
        ("bowl_unfired", 1791, "Unfired bowl", 8, 180, 1, false),
        (
            "plantpot_unfired",
            5352,
            "Unfired plant pot",
            19,
            200,
            1,
            true,
        ),
    ] {
        check(symbol, id, name, members)?;
        recipes.push(Recipe {
            input: 1761,
            input_count: 1,
            output: id,
            output_count,
            level,
            xp_tenths,
            members,
            first_ticks: 2,
            cycle_ticks: 3,
            label: name,
        });
    }
    Ok(Config {
        locations: [(14887, 1)].into(),
        recipes: std::sync::Arc::new(recipes),
        action: "shape",
        empty_message: "You don't have any suitable clay to shape.",
        animation: 883,
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
    fn pottery_shaping_five_profiles_count_four_cups_once_and_refuse_partial_capacity() {
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
            14887,
            gielinor_cache::locations::LocationType::decode(b"\x02Potter's Wheel\0\x1eUse\0\0")
                .unwrap(),
        );
        let mut obj = Symbols::new();
        for (id, symbol, name, member) in [
            (1761, "softclay", "Soft clay", false),
            (1787, "pot_unfired", "Unfired pot", false),
            (28193, "cup_unfired", "Unfired cup", true),
            (1789, "piedish_unfired", "Unfired pie dish", false),
            (1791, "bowl_unfired", "Unfired bowl", false),
            (5352, "plantpot_unfired", "Unfired plant pot", true),
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
        let loc = [("potterywheel".into(), 14887)].into();
        let seq = [("human_potterywheel".into(), 883)].into();
        let stat = [("crafting".into(), 12)].into();
        let config = pottery_shaping(&obj, &loc, &seq, &stat, &defs).unwrap();
        assert_eq!(config.recipes.len(), 5);
        assert!(!config.recipes.iter().any(|r| r.output == 4438));
        let cat = defs.inventory_catalog();
        for (recipe, fine) in config.recipes.iter().copied().zip([78, 106, 187, 225, 250]) {
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
            inv.restore_slot(
                19,
                Some(Item {
                    id: 1761,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
            let mut stats = Stats::default();
            stats.set_base(12, recipe.level as i8);
            stats.set_current(12, recipe.level as i8);
            if recipe.output_count > 1 {
                let before = (inv.clone(), stats.clone());
                assert!(
                    facility_crafting::prepare(recipe, 19, &inv, &cat, &stats, 125, true).is_err()
                );
                assert_eq!((inv.clone(), stats.clone()), before);
                for slot in [0, 1, 2] {
                    inv.restore_slot(slot, None).unwrap();
                }
            }
            let (next, next_stats, _) =
                facility_crafting::prepare(recipe, 19, &inv, &cat, &stats, 125, true).unwrap();
            assert_eq!(next_stats.fine_xp(12), fine);
            assert_eq!(
                next.slots()
                    .iter()
                    .flatten()
                    .filter(|v| v.id == recipe.output)
                    .count(),
                usize::from(recipe.output_count)
            );
            assert_eq!(next.free_space(), 0);
            if recipe.members {
                assert!(
                    facility_crafting::prepare(recipe, 19, &inv, &cat, &stats, 125, false).is_err()
                );
            }
        }
        defs.items.get_mut(&5352).unwrap().placeholder.template = 14401;
        assert!(pottery_shaping(&obj, &loc, &seq, &stat, &defs).is_err());
    }
}
