//! Eight ordinary glassblowing products share one native choice/batch dialog.
use super::{Definitions, Symbols};
use gielinor_world::{grinding, tool_crafting::Recipe};
pub fn glassblowing(
    objects: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<Recipe>, String> {
    if stats.get("crafting") != Some(&12) || sequences.get("human_glassblowing") != Some(&884) {
        return Err("Missing/aliased glassblowing stat/animation".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    let check = |symbol: &str, id: i32, name: &str, member: bool| -> Result<(), String> {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased glassblowing object {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing glassblowing object")?;
        if d.name != name
            || d.members != member
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid glassblowing object {id}"));
        }
        Ok(())
    };
    check("glassblowingpipe", 1785, "Glassblowing pipe", true)?;
    check("molten_glass", 1775, "Molten glass", true)?;
    let mut recipes = Vec::new();
    for (symbol, id, name, member, level, xp_tenths) in [
        ("beer_glass", 1919, "Beer glass", false, 1, 175),
        (
            "candle_lantern_empty",
            4527,
            "Empty candle lantern",
            true,
            4,
            190,
        ),
        ("oil_lamp_empty", 4525, "Empty oil lamp", true, 12, 250),
        ("vial_empty", 229, "Vial", false, 33, 350),
        ("fishbowl_empty", 6667, "Empty fishbowl", true, 42, 425),
        ("stafforb", 567, "Unpowered orb", true, 46, 525),
        ("bullseye_lantern_lens", 4542, "Lantern lens", true, 49, 550),
        (
            "dorgesh_lightbulb_nofilament",
            10980,
            "Empty light orb",
            true,
            87,
            700,
        ),
    ] {
        check(symbol, id, name, member)?;
        recipes.push(Recipe {
            exchange: grinding::Recipe {
                tool: 1785,
                input: 1775,
                output: id,
                members: true,
                animation: Some(884),
            },
            level,
            xp_tenths,
            ticks: 2,
            name,
            action: "blow",
        });
    }
    Ok(recipes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    #[test]
    fn eight_glass_products_are_member_skill_even_for_f2p_outputs_and_preserve_fine_xp() {
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
            (1785, "glassblowingpipe", "Glassblowing pipe", true),
            (1775, "molten_glass", "Molten glass", true),
            (1919, "beer_glass", "Beer glass", false),
            (4527, "candle_lantern_empty", "Empty candle lantern", true),
            (4525, "oil_lamp_empty", "Empty oil lamp", true),
            (229, "vial_empty", "Vial", false),
            (6667, "fishbowl_empty", "Empty fishbowl", true),
            (567, "stafforb", "Unpowered orb", true),
            (4542, "bullseye_lantern_lens", "Lantern lens", true),
            (
                10980,
                "dorgesh_lightbulb_nofilament",
                "Empty light orb",
                true,
            ),
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
        let seq = [("human_glassblowing".into(), 884)].into();
        let stat = [("crafting".into(), 12)].into();
        let recipes = glassblowing(&objects, &seq, &stat, &defs).unwrap();
        assert_eq!(recipes.len(), 8);
        let cat = defs.inventory_catalog();
        for (recipe, fine) in recipes
            .into_iter()
            .zip([218, 237, 312, 437, 531, 656, 687, 875])
        {
            let mut inv = gielinor_world::inventory::Inventory::new(
                2,
                gielinor_world::inventory::StackType::Normal,
                false,
            );
            for (slot, id) in [(0, 1785), (1, 1775)] {
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
            stats.set_base(12, recipe.level as i8);
            stats.set_current(12, recipe.level as i8);
            let job = gielinor_world::tool_crafting::Job {
                recipe,
                tool_slot: 0,
                input_slot: 1,
            };
            assert!(matches!(
                job.prepare(&inv, &cat, &stats, 125, false),
                Err(gielinor_world::tool_crafting::Error::Members)
            ));
            let p = job.prepare(&inv, &cat, &stats, 125, true).unwrap();
            assert_eq!(p.stats.fine_xp(12), fine);
            assert_eq!(p.inventory.get(0), inv.get(0));
            assert_eq!(p.inventory.get(1).unwrap().id, recipe.exchange.output);
            assert_eq!(inv.get(1).unwrap().id, 1775);
        }
        defs.items.get_mut(&10980).unwrap().members = false;
        assert!(glassblowing(&objects, &seq, &stat, &defs).is_err());
    }
}
