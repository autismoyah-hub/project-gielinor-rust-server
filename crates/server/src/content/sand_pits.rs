//! Reached selected-bucket sand gathering, without a fabricated pit operation.
use super::{Definitions, Symbols};
use gielinor_world::facility_crafting::{Config, Recipe};
pub fn sand_pits(
    objects: &Symbols,
    locations: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Config, String> {
    if stats.get("crafting") != Some(&12) || sequences.get("human_fillbucket_sandpit") != Some(&895)
    {
        return Err("Missing/aliased sand filling skill/sequence".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    for (symbol, id, name, member) in [
        ("bucket_empty", 1925, "Bucket", false),
        ("bucket_sand", 1783, "Bucket of sand", true),
    ] {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased sand vessel {symbol}"));
        }
        let d = defs.items.get(&(id as u32)).ok_or("Missing sand vessel")?;
        if d.name != name
            || d.members != member
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid sand vessel {id}"));
        }
    }
    let mut targets = std::collections::BTreeMap::new();
    for (symbol, id, size) in [
        ("sandpit", 14890, 2),
        ("prif_sandpit", 36563, 2),
        ("sandpit_2", 50733, 2),
        ("viking_sandpit_noop", 37392, 1),
    ] {
        if locations.get(symbol) != Some(&(id as i32)) {
            return Err(format!("Missing/aliased sand pit {symbol}"));
        }
        let d = defs.locations.get(&id).ok_or("Missing sand pit")?;
        if d.name != "Sand pit"
            || d.ops.iter().any(Option::is_some)
            || d.collision.width != size
            || d.collision.length != size
            || !d.multi.is_empty()
            || d.multi_default != -1
            || d.multi_varbit != -1
            || d.multi_varp != -1
        {
            return Err(format!("Invalid ordinary sand pit {id}"));
        }
        targets.insert(id, None);
    }
    Ok(Config {
        locations: targets,
        recipes: std::sync::Arc::new(vec![Recipe {
            input: 1925,
            required_inventory_tools: &[],
            secondary: None,
            byproduct: None,
            input_count: 1,
            output: 1783,
            output_count: 1,
            level: 1,
            xp_tenths: 0,
            members: true,
            first_ticks: 2,
            cycle_ticks: 3,
            failure: None,
            label: "bucket of sand",
        }]),
        action: "fill",
        empty_message: "You need an empty bucket to collect sand.",
        animation: 895,
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
    fn sand_pits_four_no_operation_targets_zero_reward_and_full_bag_credit() {
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
        defs.items.insert(
            1783,
            gielinor_cache::definitions::ItemType::decode(b"\x02Bucket of sand\0\x10\0").unwrap(),
        );
        defs.items.insert(
            1925,
            gielinor_cache::definitions::ItemType::decode(b"\x02Bucket\0\0").unwrap(),
        );
        let mut loc = Symbols::new();
        for (s, id, size) in [
            ("sandpit", 14890, 2),
            ("prif_sandpit", 36563, 2),
            ("sandpit_2", 50733, 2),
            ("viking_sandpit_noop", 37392, 1),
        ] {
            loc.insert(s.into(), id);
            let mut b = b"\x02Sand pit\0".to_vec();
            b.extend_from_slice(&[14, size, 15, size, 0]);
            defs.locations.insert(
                id as u32,
                gielinor_cache::locations::LocationType::decode(&b).unwrap(),
            );
        }
        let obj = [("bucket_empty".into(), 1925), ("bucket_sand".into(), 1783)].into();
        let seq = [("human_fillbucket_sandpit".into(), 895)].into();
        let stat = [("crafting".into(), 12)].into();
        let config = sand_pits(&obj, &loc, &seq, &stat, &defs).unwrap();
        assert_eq!(config.locations.len(), 4);
        assert!(config.locations.values().all(Option::is_none));
        let recipe = config.recipes[0];
        let mut inv = Inventory::new(28, StackType::Normal, false);
        for slot in 0..28 {
            inv.restore_slot(
                slot,
                Some(Item {
                    id: if slot == 19 { 1925 } else { 1511 },
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let stats = Stats::default();
        let cat = defs.inventory_catalog();
        let (next, reward, event) =
            facility_crafting::prepare_group(recipe, &[19], &inv, &cat, &stats, 0, true).unwrap();
        assert_eq!(next.get(19).unwrap().id, 1783);
        assert_eq!(reward, stats);
        assert!(event.is_none());
        assert!(
            facility_crafting::prepare_group(recipe, &[19], &inv, &cat, &stats, 0, false).is_err()
        );
        let mut bad = defs.inventory_catalog();
        bad.dummy.insert(1783);
        assert!(
            facility_crafting::prepare_group(recipe, &[19], &inv, &bad, &stats, 0, true).is_err()
        );
        assert_eq!(inv.get(19).unwrap().id, 1925);
        defs.locations.get_mut(&37392).unwrap().ops[0] = Some("Search".into());
        assert!(sand_pits(&obj, &loc, &seq, &stat, &defs).is_err());
    }
}
