//! Seven deterministic manual gem cuts; crushable stones and production menus are separate.
use super::{Definitions, Symbols};
use gielinor_world::{grinding, tool_crafting::Recipe};
pub fn gem_cutting(
    objects: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<Recipe>, String> {
    if stats.get("crafting") != Some(&12) {
        return Err("Missing/aliased Crafting stat".into());
    }
    let skill = defs
        .stats
        .get(&12)
        .ok_or("Missing Crafting stat definition")?;
    if skill.name != "Crafting" || skill.unreleased || skill.min_level != 1 || skill.max_level != 99
    {
        return Err("Invalid Crafting stat".into());
    }
    if sequences.get("human_cutting") != Some(&8833) {
        return Err("Missing/aliased cutting animation".into());
    }
    let checked = |symbol: &str, id: i32, name: &str, members: bool| -> Result<(), String> {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased gem object {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing gem definition")?;
        if d.name != name
            || d.members != members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid gem definition {id}"));
        }
        Ok(())
    };
    checked("chisel", 1755, "Chisel", false)?;
    let mut out = Vec::new();
    for (input_name, input, output_name, output, name, level, xp_tenths, members) in [
        (
            "uncut_sapphire",
            1623,
            "sapphire",
            1607,
            "Sapphire",
            20,
            500,
            false,
        ),
        (
            "uncut_emerald",
            1621,
            "emerald",
            1605,
            "Emerald",
            27,
            675,
            false,
        ),
        ("uncut_ruby", 1619, "ruby", 1603, "Ruby", 34, 850, false),
        (
            "uncut_diamond",
            1617,
            "diamond",
            1601,
            "Diamond",
            43,
            1075,
            false,
        ),
        (
            "uncut_dragonstone",
            1631,
            "dragonstone",
            1615,
            "Dragonstone",
            55,
            1375,
            true,
        ),
        ("uncut_onyx", 6571, "onyx", 6573, "Onyx", 67, 1675, true),
        (
            "uncut_zenyte",
            19496,
            "zenyte",
            19493,
            "Zenyte",
            89,
            500,
            true,
        ),
    ] {
        checked(
            input_name,
            input,
            &format!("Uncut {}", name.to_ascii_lowercase()),
            members,
        )?;
        checked(output_name, output, name, members)?;
        out.push(Recipe {
            exchange: grinding::Recipe {
                tool: 1755,
                input,
                output,
                members,
                animation: Some(8833),
            },
            level,
            xp_tenths,
            ticks: 1,
            name: output_name,
            action:"cut",
            output_count: 1,
            failure:None,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    #[test]
    fn seven_gem_definitions_preserve_fractional_xp_and_member_gates() {
        let mut defs = Definitions {
            inventories: BTreeMap::new(),
            locations: BTreeMap::new(),
            npcs: BTreeMap::new(),
            items: BTreeMap::new(),
            varbits: BTreeMap::new(),
            varps: BTreeMap::new(),
            stats: BTreeMap::new(),
        };
        defs.stats.insert(
            12,
            gielinor_cache::definitions::StatType::decode(b"\x01Crafting\0\0").unwrap(),
        );
        let mut objects = Symbols::new();
        let seq = [("human_cutting".into(), 8833)].into();
        let stat = [("crafting".into(), 12)].into();
        for (id, symbol, name, member) in [
            (1755, "chisel", "Chisel", false),
            (1623, "uncut_sapphire", "Uncut sapphire", false),
            (1607, "sapphire", "Sapphire", false),
            (1621, "uncut_emerald", "Uncut emerald", false),
            (1605, "emerald", "Emerald", false),
            (1619, "uncut_ruby", "Uncut ruby", false),
            (1603, "ruby", "Ruby", false),
            (1617, "uncut_diamond", "Uncut diamond", false),
            (1601, "diamond", "Diamond", false),
            (1631, "uncut_dragonstone", "Uncut dragonstone", true),
            (1615, "dragonstone", "Dragonstone", true),
            (6571, "uncut_onyx", "Uncut onyx", true),
            (6573, "onyx", "Onyx", true),
            (19496, "uncut_zenyte", "Uncut zenyte", true),
            (19493, "zenyte", "Zenyte", true),
        ] {
            objects.insert(symbol.into(), id);
            let mut raw = vec![2];
            raw.extend_from_slice(name.as_bytes());
            raw.push(0);
            if member {
                raw.push(16);
            }
            raw.push(0);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&raw).unwrap(),
            );
        }
        let recipes = gem_cutting(&objects, &seq, &stat, &defs).unwrap();
        assert_eq!(recipes.len(), 7);
        let cat = defs.inventory_catalog();
        for (r, fine) in recipes
            .into_iter()
            .zip([625, 843, 1062, 1343, 1718, 2093, 625])
        {
            let mut inv = gielinor_world::inventory::Inventory::new(
                2,
                gielinor_world::inventory::StackType::Normal,
                false,
            );
            for (slot, id) in [(0, 1755), (1, r.exchange.input)] {
                inv.restore_slot(
                    slot,
                    Some(gielinor_world::inventory::Item {
                        id,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
            let mut stats = gielinor_world::stats::Stats::default();
            stats.set_base(12, r.level as i8);
            stats.set_current(12, r.level as i8);
            let job = gielinor_world::tool_crafting::Job {
                recipe: r,
                tool_slot: 0,
                input_slot: 1,
            };
            if r.exchange.members {
                assert!(matches!(
                    job.prepare(&inv, &cat, &stats, 125, false),
                    Err(gielinor_world::tool_crafting::Error::Members)
                ));
            }
            let p = job.prepare(&inv, &cat, &stats, 125, true).unwrap();
            assert_eq!(p.stats.fine_xp(12), fine);
            assert_eq!(p.inventory.get(0), inv.get(0));
            assert_eq!(p.inventory.get(1).unwrap().id, r.exchange.output);
            assert_eq!(inv.get(1).unwrap().id, r.exchange.input);
        }
        defs.items.get_mut(&19493).unwrap().members = false;
        assert!(gem_cutting(&objects, &seq, &stat, &defs).is_err());
    }
}
