//! Hueycoatl needlework reuses captured material groups.
use super::{Definitions, Symbols};
use gielinor_world::{grinding, tool_crafting::Recipe};
const HUEY: [(&str, i32, &str, u16, u8, u16); 4] = [
    (
        "huey_vambraces",
        30082,
        "Hueycoatl hide vambraces",
        1,
        76,
        950,
    ),
    ("huey_coif", 30073, "Hueycoatl hide coif", 2, 76, 1900),
    ("huey_chaps", 30079, "Hueycoatl hide chaps", 2, 77, 1900),
    ("huey_body", 30076, "Hueycoatl hide body", 3, 78, 2850),
];
pub fn costume_needle_hueycoatl(
    objects: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<(Vec<Recipe>, Vec<u16>)>, String> {
    if stats.get("crafting") != Some(&12) || sequences.get("human_leather_crafting") != Some(&1249)
    {
        return Err("Missing fabric Crafting/animation aliases".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    let check = |symbol: &str, id: i32, name: &str, wear: bool| -> Result<(), String> {
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing fabric definition")?;
        if objects.get(symbol) != Some(&id)
            || d.name != name
            || !d.members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
            || (wear && d.inventory_ops[1].as_deref() != Some("Wear"))
        {
            return Err(format!("Invalid fabric item {id}"));
        }
        Ok(())
    };
    check("costumeneedle", 29920, "Costume needle", false)?;
    let mut groups = Vec::new();
    for (raw_symbol, raw, raw_name, rows) in
        [("huey_hide", 30085, "Hueycoatl hide", HUEY.as_slice())]
    {
        check(raw_symbol, raw, raw_name, false)?;
        let mut recipes = Vec::new();
        let mut costs = Vec::new();
        for &(symbol, out, name, cost, level, xp_tenths) in rows {
            check(symbol, out, name, true)?;
            recipes.push(Recipe {
                skill: 12,
                exchange: grinding::Recipe {
                    tool: 29920,
                    input: raw,
                    output: out,
                    members: true,
                    animation: Some(1249),
                },
                level,
                xp_tenths,
                output_count: 1,
                ticks: 2,
                name,
                action: "make",
                failure: None,
            });
            costs.push(cost);
        }
        groups.push((recipes, costs));
    }
    Ok(groups)
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        held_use::{Handler, Request, Scope, Type},
        inventory::Item,
        simulation::{PlayerData, World},
        stats::Stats,
        CoordGrid,
    };
    fn profile() -> (Definitions, Symbols, Symbols, Symbols) {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut o = Symbols::new();
        let mut rows = vec![
            ("costumeneedle", 29920, "Costume needle", false),
            ("huey_hide", 30085, "Hueycoatl hide", false),
        ];
        rows.extend(HUEY.iter().map(|&(s, id, n, _, _, _)| (s, id, n, true)));
        for (s, id, n, wear) in rows {
            let mut item = gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();
            item.name = n.into();
            item.members = true;
            if wear {
                item.inventory_ops[1] = Some("Wear".into());
            }
            d.items.insert(id as u32, item);
            o.insert(s.into(), id);
        }
        d.stats.insert(
            12,
            gielinor_cache::definitions::StatType::decode(b"\x01Crafting\0\0").unwrap(),
        );
        (
            d,
            o,
            [("human_leather_crafting".into(), 1249)].into(),
            [("crafting".into(), 12)].into(),
        )
    }
    #[test]
    fn costume_needle_hueycoatl_exact_four_products_counts_and_late_invalid_output() {
        let (mut d, o, s, t) = profile();
        let groups = costume_needle_hueycoatl(&o, &s, &t, &d).unwrap();
        assert_eq!(groups[0].1, vec![1, 2, 2, 3]);
        assert_eq!(groups.iter().map(|g| g.0.len()).sum::<usize>(), 4);
        d.items.get_mut(&30076).unwrap().stackable = true;
        assert!(costume_needle_hueycoatl(&o, &s, &t, &d).is_err());
    }
    #[test]
    fn costume_needle_hueycoatl_actual_captured_large_groups_full_bag_both_orders_and_unused_remainder(
    ) {
        let (d, o, s, t) = profile();
        for (recipes, costs) in costume_needle_hueycoatl(&o, &s, &t, &d).unwrap() {
            let recipes = std::sync::Arc::new(recipes);
            let costs = std::sync::Arc::new(costs);
            for index in 0..recipes.len() {
                for reverse in [false, true] {
                    let recipe = recipes[index];
                    let raw = recipe.exchange.input;
                    let cost = usize::from(costs[index]);
                    let mut w = World::default();
                    let mut stats = Stats::default();
                    stats.set_base(12, 1);
                    stats.set_current(12, 78);
                    let mut data =
                        PlayerData::new(1, "Fabric maker".into(), CoordGrid::ZERO, stats);
                    data.xp_rate_hundreds = 125;
                    for slot in 0..28 {
                        data.inventory
                            .restore_slot(
                                slot,
                                Some(Item {
                                    id: raw,
                                    count: 1,
                                    vars: 0,
                                }),
                            )
                            .unwrap();
                    }
                    data.inventory
                        .restore_slot(
                            4,
                            Some(Item {
                                id: 29920,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    w.queue_login(1, data).unwrap();
                    w.tick().unwrap();
                    w.tick().unwrap();
                    let actor = w.actors().next().unwrap();
                    w.set_membership(actor, true).unwrap();
                    for id in [29920, raw] {
                        w.held_use_types.insert(id, Type { id, content: -1 });
                    }
                    w.held_use_handlers
                        .register(
                            Scope::Types,
                            29920,
                            raw,
                            Handler::ToolCraftingMaterialChoices {
                                recipes: recipes.clone(),
                                input_counts: costs.clone(),
                                cycle_ticks: 3,
                            },
                        )
                        .unwrap();
                    let held = if reverse {
                        Request {
                            selected_slot: 17,
                            selected_id: raw,
                            target_slot: 4,
                            target_id: 29920,
                        }
                    } else {
                        Request {
                            selected_slot: 4,
                            selected_id: 29920,
                            target_slot: 17,
                            target_id: raw,
                        }
                    };
                    w.finish_held_use(actor, held).unwrap();
                    let request = w.take_tool_crafting_request(actor).unwrap().unwrap();
                    assert_eq!(
                        request.available_for(index, &w.data(actor).unwrap().inventory),
                        27 / cost
                    );
                    let wanted = 2.min(27 / cost);
                    let start = w.clock.0;
                    w.start_tool_crafting_choice(actor, request, index, 2)
                        .unwrap();
                    let mut commits = Vec::new();
                    let mut old = 0;
                    for _ in 0..8 {
                        w.tick().unwrap();
                        let now = w
                            .data(actor)
                            .unwrap()
                            .inventory
                            .slots()
                            .iter()
                            .flatten()
                            .filter(|i| i.id == recipe.exchange.output)
                            .count();
                        if now != old {
                            commits.push(w.clock.0 - 1);
                        }
                        old = now;
                    }
                    let data = w.data(actor).unwrap();
                    assert_eq!(
                        commits,
                        (0..wanted)
                            .map(|n| start + 2 + 3 * n as i32)
                            .collect::<Vec<_>>()
                    );
                    assert_eq!(old, wanted);
                    assert_eq!(
                        data.inventory
                            .slots()
                            .iter()
                            .flatten()
                            .filter(|i| i.id == raw)
                            .count(),
                        27 - wanted * cost
                    );
                    assert_eq!(data.inventory.free_space(), wanted * (cost - 1));
                    assert_eq!(data.inventory.get(4).unwrap().id, 29920);
                    assert_eq!(
                        data.stats.fine_xp(12),
                        (i32::from(recipe.xp_tenths) * 125 / 100) * wanted as i32
                    );
                }
            }
        }
    }
}
