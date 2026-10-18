//! Splitbark mixed-material plans reuse retained-tool production.
use super::{Definitions, Symbols};
use gielinor_world::{grinding, tool_crafting::Recipe};
const PRODUCTS: [(&str, i32, &str, u16, u8, u16); 5] = [
    (
        "splitbark_gauntlets",
        3391,
        "Splitbark gauntlets",
        1,
        60,
        620,
    ),
    ("splitbark_greaves", 3393, "Splitbark boots", 1, 60, 620),
    ("splitbark_helm", 3385, "Splitbark helm", 2, 61, 1240),
    ("splitbark_legs", 3389, "Splitbark legs", 3, 62, 1860),
    ("splitbark_body", 3387, "Splitbark body", 4, 62, 2480),
];
pub fn costume_needle_splitbark(
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
    check("hollow_bark", 3239, "Bark", false)?;
    check("fine_cloth", 3470, "Fine cloth", false)?;
    let mut recipes = Vec::new();
    let mut materials = Vec::new();
    for &(symbol, out, name, cost, level, xp_tenths) in &PRODUCTS {
        check(symbol, out, name, true)?;
        recipes.push(Recipe {
            skill: 12,
            exchange: grinding::Recipe {
                tool: 29920,
                input: 3239,
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
        materials.push(vec![(3239, cost), (3470, cost)]);
    }
    Ok((recipes, materials))
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
            ("hollow_bark", 3239, "Bark", false),
            ("fine_cloth", 3470, "Fine cloth", false),
        ];
        rows.extend(PRODUCTS.iter().map(|&(s, id, n, _, _, _)| (s, id, n, true)));
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
    fn costume_needle_splitbark_profile_exact_five_and_late_refusal() {
        let (mut d, o, s, t) = profile();
        let (recipes, costs) = costume_needle_splitbark(&o, &s, &t, &d).unwrap();
        assert_eq!(recipes.len(), 5);
        assert_eq!(
            costs,
            vec![
                vec![(3239, 1), (3470, 1)],
                vec![(3239, 1), (3470, 1)],
                vec![(3239, 2), (3470, 2)],
                vec![(3239, 3), (3470, 3)],
                vec![(3239, 4), (3470, 4)]
            ]
        );
        d.items.get_mut(&3387).unwrap().stackable = true;
        assert!(costume_needle_splitbark(&o, &s, &t, &d).is_err());
    }
    fn fixture() -> (
        World,
        gielinor_world::simulation::ActorId,
        std::sync::Arc<Vec<Recipe>>,
        gielinor_world::tool_crafting_materials::Specifications,
    ) {
        let (d, o, s, t) = profile();
        let (recipes, specs) = costume_needle_splitbark(&o, &s, &t, &d).unwrap();
        let recipes = std::sync::Arc::new(recipes);
        let specs = std::sync::Arc::new(specs);
        let mut w = World::default();
        let mut stats = Stats::default();
        stats.set_base(12, 1);
        stats.set_current(12, 62);
        let mut data = PlayerData::new(1, "Splitbark maker".into(), CoordGrid::ZERO, stats);
        data.xp_rate_hundreds = 125;
        for slot in 0..28 {
            data.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id: if slot < 14 { 3239 } else { 3470 },
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
        for id in [29920, 3239, 3470] {
            w.held_use_types.insert(id, Type { id, content: -1 });
        }
        for (a, b) in [(29920, 3239), (29920, 3470), (3239, 3470)] {
            w.held_use_handlers
                .register(
                    Scope::Types,
                    a,
                    b,
                    Handler::ToolCraftingMixedChoices {
                        recipes: recipes.clone(),
                        materials: specs.clone(),
                        cycle_ticks: 3,
                    },
                )
                .unwrap();
        }
        (w, actor, recipes, specs)
    }
    fn held(route: usize, reverse: bool) -> Request {
        let ((a, ai), (b, bi)) = [
            ((4, 29920), (7, 3239)),
            ((4, 29920), (23, 3470)),
            ((7, 3239), (23, 3470)),
        ][route];
        if reverse {
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
        }
    }
    fn count(w: &World, a: gielinor_world::simulation::ActorId, id: i32) -> usize {
        w.data(a)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .filter(|v| v.id == id)
            .count()
    }
    #[test]
    fn costume_needle_splitbark_actual_all_five_all_six_dispatch_directions_fullbag() {
        for index in 0..5 {
            for route in 0..3 {
                for reverse in [false, true] {
                    let (mut w, a, recipes, _) = fixture();
                    let recipe = recipes[index];
                    let cost = usize::from(PRODUCTS[index].3);
                    w.finish_held_use(a, held(route, reverse)).unwrap();
                    let req = w.take_tool_crafting_request(a).unwrap().unwrap();
                    assert_eq!(
                        req.available_for(index, &w.data(a).unwrap().inventory),
                        13 / cost
                    );
                    let start = w.clock.0;
                    w.start_tool_crafting_choice(a, req, index, 2).unwrap();
                    let mut times = Vec::new();
                    let mut old = 0;
                    for _ in 0..8 {
                        w.tick().unwrap();
                        let n = count(&w, a, recipe.exchange.output);
                        if n != old {
                            times.push(w.clock.0 - 1);
                        }
                        old = n;
                    }
                    assert_eq!(times, vec![start + 2, start + 5]);
                    assert_eq!(old, 2);
                    assert_eq!(count(&w, a, 3239), 13 - 2 * cost);
                    assert_eq!(count(&w, a, 3470), 14 - 2 * cost);
                    assert_eq!(
                        w.data(a).unwrap().inventory.free_space(),
                        2 * (2 * cost - 1)
                    );
                    assert_eq!(w.data(a).unwrap().inventory.get(4).unwrap().id, 29920);
                    assert_eq!(
                        w.data(a).unwrap().stats.fine_xp(12),
                        2 * (i32::from(recipe.xp_tenths) * 125 / 100)
                    );
                }
            }
        }
    }
    #[test]
    fn costume_needle_splitbark_selected_secondary_and_late_captured_secondary_fail_closed() {
        for case in 0..4 {
            let (mut w, a, _, _) = fixture();
            if case == 0 {
                w.data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(4, None)
                    .unwrap();
                assert!(w.finish_held_use(a, held(2, false)).is_err());
                continue;
            }
            w.finish_held_use(a, held(1, false)).unwrap();
            let req = w.take_tool_crafting_request(a).unwrap().unwrap();
            if case == 1 {
                w.data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(23, None)
                    .unwrap();
                assert!(w.start_tool_crafting_choice(a, req, 0, 2).is_err());
                continue;
            }
            w.start_tool_crafting_choice(a, req, 0, 2).unwrap();
            for _ in 0..3 {
                w.tick().unwrap();
            }
            assert_eq!(count(&w, a, 3391), 1);
            if case == 2 {
                w.data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        14,
                        Some(Item {
                            id: 3470,
                            count: 1,
                            vars: 9,
                        }),
                    )
                    .unwrap();
            } else {
                w.set_membership(a, false).unwrap();
            }
            let before = w.data(a).unwrap().inventory.clone();
            let xp = w.data(a).unwrap().stats.fine_xp(12);
            for _ in 0..5 {
                w.tick().unwrap();
            }
            assert_eq!(w.data(a).unwrap().inventory, before);
            assert_eq!(w.data(a).unwrap().stats.fine_xp(12), xp);
            assert_eq!(count(&w, a, 3391), 1);
        }
    }
}
