//! Standard-chisel semiprecious cuts use pinned Wiki curves rather than donor placeholder rates.
use super::{Definitions, Symbols};
use gielinor_world::{
    grinding,
    tool_crafting::{Failure, Recipe},
};
pub fn fallible_gems(
    objects: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<Recipe>, String> {
    if stats.get("crafting") != Some(&12) {
        return Err("Missing/aliased fallible-gem stat".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    let check = |symbol: &str, id: i32, name: &str, member: bool| -> Result<(), String> {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased semiprecious object {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing semiprecious item")?;
        if d.name != name
            || d.members != member
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid semiprecious item {id}"));
        }
        Ok(())
    };
    check("chisel", 1755, "Chisel", false)?;
    check("crushed_gemstone", 1633, "Crushed gem", true)?;
    let mut out = Vec::new();
    for (
        input_symbol,
        input,
        output_symbol,
        output,
        name,
        level,
        xp_tenths,
        fail_xp,
        low,
        high,
        anim_symbol,
        animation,
    ) in [
        (
            "uncut_opal",
            1625,
            "opal",
            1609,
            "Opal",
            1,
            150,
            38,
            128,
            250,
            "human_opalcutting",
            890,
        ),
        (
            "uncut_jade",
            1627,
            "jade",
            1611,
            "Jade",
            13,
            200,
            50,
            100,
            245,
            "human_jadecutting",
            891,
        ),
        (
            "uncut_red_topaz",
            1629,
            "red_topaz",
            1613,
            "Red topaz",
            16,
            250,
            63,
            90,
            240,
            "human_redtopazcutting",
            892,
        ),
    ] {
        check(
            input_symbol,
            input,
            &format!("Uncut {}", name.to_ascii_lowercase()),
            true,
        )?;
        check(output_symbol, output, name, true)?;
        if sequences.get(anim_symbol) != Some(&animation) {
            return Err(format!("Missing/aliased gem sequence {anim_symbol}"));
        }
        out.push(Recipe {
            exchange: grinding::Recipe {
                tool: 1755,
                input,
                output,
                members: true,
                animation: Some(animation as u16),
            },
            level,
            xp_tenths,
            ticks: 1,
            name,
            action: "cut",
            failure: Some(Failure {
                output: 1633,
                xp_tenths: fail_xp,
                low,
                high,
                message: "You accidentally crush the gem.",
            }),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    struct Draw(u32);
    impl gielinor_world::npc::RandomSource for Draw {
        fn below(&mut self, n: u32) -> u32 {
            assert_eq!(n, 256);
            self.0
        }
    }
    #[test]
    fn wiki_semiprecious_boundaries_failure_fine_xp_and_exact_variants_are_bound() {
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
            (1625, "uncut_opal", "Uncut opal", true),
            (1609, "opal", "Opal", true),
            (1627, "uncut_jade", "Uncut jade", true),
            (1611, "jade", "Jade", true),
            (1629, "uncut_red_topaz", "Uncut red topaz", true),
            (1613, "red_topaz", "Red topaz", true),
            (1633, "crushed_gemstone", "Crushed gem", true),
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
        let seq = [
            ("human_opalcutting".into(), 890),
            ("human_jadecutting".into(), 891),
            ("human_redtopazcutting".into(), 892),
        ]
        .into();
        let stat = [("crafting".into(), 12)].into();
        let recipes = fallible_gems(&objects, &seq, &stat, &defs).unwrap();
        assert_eq!(recipes.len(), 3);
        let cat = defs.inventory_catalog();
        for (recipe, (minimum, at99, success_fine, failed_fine)) in recipes.into_iter().zip([
            (129, 251, 187, 47),
            (119, 246, 250, 62),
            (114, 241, 312, 78),
        ]) {
            let mut inv = gielinor_world::inventory::Inventory::new(
                2,
                gielinor_world::inventory::StackType::Normal,
                false,
            );
            for (slot, id) in [(0, 1755), (1, recipe.exchange.input)] {
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
            let job = gielinor_world::tool_crafting::Job {
                recipe,
                tool_slot: 0,
                input_slot: 1,
            };
            for (level, threshold) in [(recipe.level, minimum), (99, at99)] {
                let mut stats = gielinor_world::stats::Stats::default();
                stats.set_base(12, level as i8);
                stats.set_current(12, level as i8);
                for (roll, success, fine, id) in [
                    (threshold - 1, true, success_fine, recipe.exchange.output),
                    (threshold, false, failed_fine, 1633),
                ] {
                    let (prepared, actual) = job
                        .resolve(&inv, &cat, &stats, 125, true, &mut Draw(roll))
                        .unwrap();
                    assert_eq!(actual, success);
                    assert_eq!(prepared.stats.fine_xp(12), fine);
                    assert_eq!(prepared.inventory.get(1).unwrap().id, id);
                    assert_eq!(prepared.inventory.get(0), inv.get(0));
                }
            }
        }
        defs.items.get_mut(&1633).unwrap().cert.template = 799;
        assert!(fallible_gems(&objects, &seq, &stat, &defs).is_err());
    }
}
