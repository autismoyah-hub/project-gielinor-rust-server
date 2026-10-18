//! Sand and soda ash at attested ordinary furnaces; normal Smelt remains Smithing.
use super::{Definitions, Symbols};
use gielinor_world::facility_crafting::{Config, Recipe};
pub fn glass_smelting(
    objects: &Symbols,
    locations: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Config, String> {
    if stats.get("crafting") != Some(&12) || sequences.get("human_furnace") != Some(&899) {
        return Err("Missing/aliased glass furnace skill/animation".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    for (symbol, id, name, member) in [
        ("bucket_sand", 1783, "Bucket of sand", true),
        ("soda_ash", 1781, "Soda ash", true),
        ("molten_glass", 1775, "Molten glass", true),
        ("bucket_empty", 1925, "Bucket", false),
    ] {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased glass material {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing glass material")?;
        if d.name != name
            || d.members != member
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid glass material {id}"));
        }
    }
    let mut targets = std::collections::BTreeMap::new();
    for (symbol, id, width) in [("furnace", 2030, 3), ("viking_furnace", 4304, 2)] {
        if locations.get(symbol) != Some(&(id as i32)) {
            return Err(format!("Missing/aliased glass furnace {symbol}"));
        }
        let d = defs.locations.get(&id).ok_or("Missing glass furnace")?;
        if d.name != "Furnace"
            || d.category != 215
            || d.ops[1].as_deref() != Some("Smelt")
            || d.collision.width != width
            || d.collision.length != 3
            || !d.multi.is_empty()
            || d.multi_default != -1
            || d.multi_varbit != -1
            || d.multi_varp != -1
        {
            return Err(format!("Invalid ordinary glass furnace {id}"));
        }
        targets.insert(id, None);
    }
    Ok(Config {
        locations: targets,
        recipes: std::sync::Arc::new(vec![Recipe {
            input: 1783,
            input_count: 1,
            required_inventory_tools: &[],
            secondary: Some(1781),
            output: 1775,
            output_count: 1,
            byproduct: Some(1925),
            level: 1,
            xp_tenths: 200,
            members: true,
            first_ticks: 2,
            cycle_ticks: 3,
            failure: None,
            label: "molten glass",
        }]),
        action: "smelt",
        empty_message: "You need sand and soda ash to make glass.",
        animation: 899,
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
    fn glass_smelting_mixed_material_and_returned_bucket_stage_atomically_with_precise_xp() {
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
        let mut objects = Symbols::new();
        for (symbol, id, name, member) in [
            ("bucket_sand", 1783, "Bucket of sand", true),
            ("soda_ash", 1781, "Soda ash", true),
            ("molten_glass", 1775, "Molten glass", true),
            ("bucket_empty", 1925, "Bucket", false),
        ] {
            objects.insert(symbol.into(), id);
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
        let mut loc = Symbols::new();
        for (symbol, id, width) in [("furnace", 2030, 3), ("viking_furnace", 4304, 2)] {
            loc.insert(symbol.into(), id);
            let mut b = b"\x02Furnace\0".to_vec();
            b.extend_from_slice(&[14, width, 15, 3, 61, 0, 215, 31]);
            b.extend_from_slice(b"Smelt\0\0");
            defs.locations.insert(
                id as u32,
                gielinor_cache::locations::LocationType::decode(&b).unwrap(),
            );
        }
        let seq = [("human_furnace".into(), 899)].into();
        let stat = [("crafting".into(), 12)].into();
        let config = glass_smelting(&objects, &loc, &seq, &stat, &defs).unwrap();
        assert!(config.locations.values().all(Option::is_none));
        let recipe = config.recipes[0];
        let mut inv = Inventory::new(28, StackType::Normal, false);
        for slot in 0..28 {
            inv.restore_slot(
                slot,
                Some(Item {
                    id: if slot == 19 {
                        1783
                    } else if slot == 4 {
                        1781
                    } else {
                        1511
                    },
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let stats = Stats::default();
        let cat = defs.inventory_catalog();
        let (next, reward, _) =
            facility_crafting::prepare_group(recipe, &[19, 4], &inv, &cat, &stats, 125, true)
                .unwrap();
        assert_eq!(reward.fine_xp(12), 250);
        assert_eq!(next.free_space(), 0);
        for id in [1775, 1925] {
            assert_eq!(
                next.slots().iter().flatten().filter(|i| i.id == id).count(),
                1
            );
        }
        let before = (inv.clone(), stats.clone());
        let mut bad = defs.inventory_catalog();
        bad.dummy.insert(1925);
        assert!(
            facility_crafting::prepare_group(recipe, &[19, 4], &inv, &bad, &stats, 125, true)
                .is_err()
        );
        assert!(
            facility_crafting::prepare_group(recipe, &[19], &inv, &cat, &stats, 125, true).is_err()
        );
        assert!(
            facility_crafting::prepare_group(recipe, &[19, 19], &inv, &cat, &stats, 125, true)
                .is_err()
        );
        assert!(
            facility_crafting::prepare_group(recipe, &[19, 4], &inv, &cat, &stats, 125, false)
                .is_err()
        );
        assert_eq!((inv, stats), before);
        defs.locations.get_mut(&2030).unwrap().category = 0;
        assert!(glass_smelting(&objects, &loc, &seq, &stat, &defs).is_err());
    }
}
