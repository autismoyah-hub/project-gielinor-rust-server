//! Nine legacy shell carvings through the existing manual Crafting transaction.
use super::{Definitions, Symbols};
use gielinor_world::{grinding, tool_crafting::Recipe};
pub fn snelm_carving(
    objects: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<Recipe>, String> {
    if stats.get("crafting") != Some(&12) || sequences.get("human_snailshellcutting") != Some(&1280)
    {
        return Err("Missing/aliased shell carving stat/animation".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    let check = |symbol: &str, id: i32, name: &str, members: bool| -> Result<(), String> {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased carving object {symbol}"));
        }
        let d = defs.items.get(&(id as u32)).ok_or("Missing carving item")?;
        if d.name != name
            || d.members != members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid carving item {id}"));
        }
        Ok(())
    };
    check("chisel", 1755, "Chisel", false)?;
    let mut recipes = Vec::new();
    for (input, input_symbol, input_name, output, output_symbol, name) in [
        (
            3345,
            "shellround_swamp",
            "Blamish myre shell",
            3327,
            "snelm_round_swamp",
            "Myre snelm",
        ),
        (
            3347,
            "shellround_red+black",
            "Blamish red shell",
            3329,
            "snelm_round_red+black",
            "Blood'n'tar snelm",
        ),
        (
            3349,
            "shellround_yellow",
            "Blamish ochre shell",
            3331,
            "snelm_round_yellow",
            "Ochre snelm",
        ),
        (
            3351,
            "shellround_blue",
            "Blamish blue shell",
            3333,
            "snelm_round_blue",
            "Bruise blue snelm",
        ),
        (
            3353,
            "shellround_orange",
            "Blamish bark shell",
            3335,
            "snelm_round_orange",
            "Broken bark snelm",
        ),
        (
            3355,
            "shellpoint_swamp",
            "Blamish myre shell",
            3337,
            "snelm_point_swamp",
            "Pointed myre snelm",
        ),
        (
            3357,
            "shellpoint_red+black",
            "Blamish red shell",
            3339,
            "snelm_point_red+black",
            "Pointed blood'n'tar snelm",
        ),
        (
            3359,
            "shellpoint_yellow",
            "Blamish ochre shell",
            3341,
            "snelm_point_yellow",
            "Pointed ochre snelm",
        ),
        (
            3361,
            "shellpoint_blue",
            "Blamish blue shell",
            3343,
            "snelm_point_blue",
            "Pointed bruise blue snelm",
        ),
    ] {
        check(input_symbol, input, input_name, true)?;
        check(output_symbol, output, name, true)?;
        recipes.push(Recipe {
            exchange: grinding::Recipe {
                tool: 1755,
                input,
                output,
                members: true,
                animation: Some(1280),
            },
            level: 15,
            xp_tenths: 325,
            ticks: 2,
            name,
        });
    }
    Ok(recipes)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    #[test]
    fn nine_round_pointed_and_bark_shells_preserve_selected_full_bag_and_fractional_rewards() {
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
        for (id, symbol, name, member) in [
            (1755, "chisel", "Chisel", false),
            (3345, "shellround_swamp", "Blamish myre shell", true),
            (3327, "snelm_round_swamp", "Myre snelm", true),
            (3347, "shellround_red+black", "Blamish red shell", true),
            (3329, "snelm_round_red+black", "Blood'n'tar snelm", true),
            (3349, "shellround_yellow", "Blamish ochre shell", true),
            (3331, "snelm_round_yellow", "Ochre snelm", true),
            (3351, "shellround_blue", "Blamish blue shell", true),
            (3333, "snelm_round_blue", "Bruise blue snelm", true),
            (3353, "shellround_orange", "Blamish bark shell", true),
            (3335, "snelm_round_orange", "Broken bark snelm", true),
            (3355, "shellpoint_swamp", "Blamish myre shell", true),
            (3337, "snelm_point_swamp", "Pointed myre snelm", true),
            (3357, "shellpoint_red+black", "Blamish red shell", true),
            (
                3339,
                "snelm_point_red+black",
                "Pointed blood'n'tar snelm",
                true,
            ),
            (3359, "shellpoint_yellow", "Blamish ochre shell", true),
            (3341, "snelm_point_yellow", "Pointed ochre snelm", true),
            (3361, "shellpoint_blue", "Blamish blue shell", true),
            (3343, "snelm_point_blue", "Pointed bruise blue snelm", true),
        ] {
            objects.insert(symbol.into(), id);
            let mut raw = vec![2];
            raw.extend_from_slice(name.as_bytes());
            raw.push(0);
            if member {
                raw.push(16)
            }
            raw.push(0);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&raw).unwrap(),
            );
        }
        let seq = [("human_snailshellcutting".into(), 1280)].into();
        let stat = [("crafting".into(), 12)].into();
        let recipes = snelm_carving(&objects, &seq, &stat, &defs).unwrap();
        assert_eq!(recipes.len(), 9);
        let catalog = defs.inventory_catalog();
        for recipe in recipes {
            for (rate, fine) in [(100, 325), (125, 406), (500, 1625)] {
                let mut inv = gielinor_world::inventory::Inventory::new(
                    28,
                    gielinor_world::inventory::StackType::Normal,
                    false,
                );
                for slot in 0..28 {
                    inv.restore_slot(
                        slot,
                        Some(gielinor_world::inventory::Item {
                            id: 1511,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                inv.restore_slot(
                    0,
                    Some(gielinor_world::inventory::Item {
                        id: 1755,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
                inv.restore_slot(
                    27,
                    Some(gielinor_world::inventory::Item {
                        id: recipe.exchange.input,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
                let mut stats = gielinor_world::stats::Stats::default();
                stats.set_base(12, 15);
                stats.set_current(12, 15);
                let job = gielinor_world::tool_crafting::Job {
                    recipe,
                    tool_slot: 0,
                    input_slot: 27,
                };
                assert!(matches!(
                    job.prepare(&inv, &catalog, &stats, rate, false),
                    Err(gielinor_world::tool_crafting::Error::Members)
                ));
                let prepared = job.prepare(&inv, &catalog, &stats, rate, true).unwrap();
                assert_eq!(prepared.stats.fine_xp(12), fine);
                assert_eq!(
                    prepared.inventory.get(27).unwrap().id,
                    recipe.exchange.output
                );
                assert_eq!(prepared.inventory.get(0), inv.get(0));
                assert_eq!(prepared.inventory.get(26), inv.get(26));
                assert_eq!(inv.get(27).unwrap().id, recipe.exchange.input);
            }
        }
        defs.items.get_mut(&3343).unwrap().cert.template = 799;
        assert!(snelm_carving(&objects, &seq, &stat, &defs).is_err());
    }
}
