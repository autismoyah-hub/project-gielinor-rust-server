//! Mixed-hide clothing shares captured mixed-material production.
use super::{Definitions, Symbols};
use gielinor_world::{grinding, tool_crafting::Recipe};
const PRODUCTS: [(&str, i32, &str, i32, u16, u8, u16); 4] = [
    ("hide_cape", 29289, "Mixed hide cape", 29218, 1, 68, 620),
    ("hide_boots", 29286, "Mixed hide boots", 29177, 1, 69, 750),
    ("hide_legs", 29283, "Mixed hide legs", 29163, 3, 71, 2100),
    ("hide_top", 29280, "Mixed hide top", 29177, 2, 72, 1500),
];
pub fn costume_needle_mixedhide(
    objects: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<(Vec<Recipe>, Vec<Vec<(i32, u16)>>), String> {
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
    check("hg_mixedhide_base", 29292, "Mixed hide base", false)?;
    check("varlamore_jaguar_fur", 29218, "Jaguar fur", false)?;
    check(
        "hunting_antelopesun_fur",
        29177,
        "Sunlight antelope fur",
        false,
    )?;
    check("hunting_fennecfox_fur", 29163, "Fox fur", false)?;
    let mut recipes = Vec::new();
    let mut materials = Vec::new();
    for &(symbol, out, name, secondary, cost, level, xp_tenths) in &PRODUCTS {
        check(symbol, out, name, true)?;
        recipes.push(Recipe {
            skill: 12,
            exchange: grinding::Recipe {
                tool: 29920,
                input: 29292,
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
        materials.push(vec![(29292, 1), (secondary, cost)]);
    }
    Ok((recipes, materials))
}

/// Group menus by the actual clicked pair, matching source per-pair candidates.
pub fn costume_needle_mixedhide_registrations(
    recipes: &[Recipe],
    specs: &[Vec<(i32, u16)>],
) -> Vec<(i32, i32, Vec<Recipe>, Vec<Vec<(i32, u16)>>)> {
    let mut groups = Vec::new();
    for (a, b) in [
        (29920, 29292),
        (29920, 29218),
        (29292, 29218),
        (29920, 29177),
        (29292, 29177),
        (29920, 29163),
        (29292, 29163),
    ] {
        let mut rs = Vec::new();
        let mut ss = Vec::new();
        for (recipe, spec) in recipes.iter().zip(specs) {
            if [a, b]
                .iter()
                .all(|id| *id == 29920 || spec.iter().any(|v| v.0 == *id))
            {
                rs.push(*recipe);
                ss.push(spec.clone());
            }
        }
        groups.push((a, b, rs, ss));
    }
    groups
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
            ("hg_mixedhide_base", 29292, "Mixed hide base", false),
            ("varlamore_jaguar_fur", 29218, "Jaguar fur", false),
            (
                "hunting_antelopesun_fur",
                29177,
                "Sunlight antelope fur",
                false,
            ),
            ("hunting_fennecfox_fur", 29163, "Fox fur", false),
        ];
        rows.extend(
            PRODUCTS
                .iter()
                .map(|&(s, id, n, _, _, _, _)| (s, id, n, true)),
        );
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
    fn costume_needle_mixedhide_exact_four_recipes_seven_filtered_pairs_and_late_profile_refusal() {
        let (mut d, o, s, t) = profile();
        let (rs, ss) = costume_needle_mixedhide(&o, &s, &t, &d).unwrap();
        assert_eq!(
            ss,
            vec![
                vec![(29292, 1), (29218, 1)],
                vec![(29292, 1), (29177, 1)],
                vec![(29292, 1), (29163, 3)],
                vec![(29292, 1), (29177, 2)]
            ]
        );
        let groups = costume_needle_mixedhide_registrations(&rs, &ss);
        assert_eq!(
            groups.iter().map(|v| v.2.len()).collect::<Vec<_>>(),
            vec![4, 1, 1, 2, 2, 1, 1]
        );
        d.items.get_mut(&29280).unwrap().stackable = true;
        assert!(costume_needle_mixedhide(&o, &s, &t, &d).is_err());
    }
    #[test]
    fn costume_needle_mixedhide_actual_asymmetric_groups_all_four_all_six_routes_fullbag() {
        let (d, o, s, t) = profile();
        let (recipes, specs) = costume_needle_mixedhide(&o, &s, &t, &d).unwrap();
        for (index, recipe) in recipes.iter().enumerate() {
            let secondary = specs[index][1].0;
            let cost = usize::from(specs[index][1].1);
            for route in 0..3 {
                for reverse in [false, true] {
                    let mut w = World::default();
                    let mut stats = Stats::default();
                    stats.set_base(12, 1);
                    stats.set_current(12, 72);
                    let mut data =
                        PlayerData::new(1, "Mixed hide maker".into(), CoordGrid::ZERO, stats);
                    data.xp_rate_hundreds = 125;
                    for slot in 0..28 {
                        data.inventory
                            .restore_slot(
                                slot,
                                Some(Item {
                                    id: if slot < 14 { 29292 } else { secondary },
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
                    for id in [29920, 29292, secondary] {
                        w.held_use_types.insert(id, Type { id, content: -1 });
                    }
                    for (a, b, rs, ss) in costume_needle_mixedhide_registrations(&recipes, &specs) {
                        w.held_use_handlers
                            .register(
                                Scope::Types,
                                a,
                                b,
                                Handler::ToolCraftingMixedChoices {
                                    recipes: std::sync::Arc::new(rs),
                                    materials: std::sync::Arc::new(ss),
                                    cycle_ticks: 3,
                                },
                            )
                            .unwrap();
                    }
                    let ((a, ai), (b, bi)) = [
                        ((4, 29920), (7, 29292)),
                        ((4, 29920), (23, secondary)),
                        ((7, 29292), (23, secondary)),
                    ][route];
                    let held = if reverse {
                        Request {
                            selected_slot: b,
                            selected_id: bi,
                            target_slot: a,
                            target_id: ai,
                        }
                    } else {
                        Request {
                            selected_slot: a,
                            selected_id: ai,
                            target_slot: b,
                            target_id: bi,
                        }
                    };
                    w.finish_held_use(actor, held).unwrap();
                    let req = w.take_tool_crafting_request(actor).unwrap().unwrap();
                    if route == 0 {
                        assert_eq!(req.recipes.len(), 4);
                    } else {
                        assert!(req
                            .material_specs
                            .as_ref()
                            .unwrap()
                            .iter()
                            .all(|spec| spec.iter().any(|v| v.0 == secondary)));
                        assert_eq!(req.recipes.len(), if secondary == 29177 { 2 } else { 1 });
                    }
                    let selected = req
                        .recipes
                        .iter()
                        .position(|v| v.exchange.output == recipe.exchange.output)
                        .unwrap();
                    assert_eq!(
                        req.available_for(selected, &w.data(actor).unwrap().inventory),
                        13.min(14 / cost)
                    );
                    let start = w.clock.0;
                    w.start_tool_crafting_choice(actor, req, selected, 2)
                        .unwrap();
                    let mut old = 0;
                    let mut times = Vec::new();
                    for _ in 0..8 {
                        w.tick().unwrap();
                        let n = w
                            .data(actor)
                            .unwrap()
                            .inventory
                            .slots()
                            .iter()
                            .flatten()
                            .filter(|v| v.id == recipe.exchange.output)
                            .count();
                        if n != old {
                            times.push(w.clock.0 - 1);
                        }
                        old = n;
                    }
                    let data = w.data(actor).unwrap();
                    let count = |id| {
                        data.inventory
                            .slots()
                            .iter()
                            .flatten()
                            .filter(|v| v.id == id)
                            .count()
                    };
                    assert_eq!(times, vec![start + 2, start + 5]);
                    assert_eq!(old, 2);
                    assert_eq!(count(29292), 11);
                    assert_eq!(count(secondary), 14 - 2 * cost);
                    assert_eq!(data.inventory.free_space(), 2 * cost);
                    assert_eq!(data.inventory.get(4).unwrap().id, 29920);
                    assert_eq!(
                        data.stats.fine_xp(12),
                        2 * (i32::from(recipe.xp_tenths) * 125 / 100)
                    );
                }
            }
        }
    }
}
