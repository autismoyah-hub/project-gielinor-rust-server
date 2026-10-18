//! Add two-/three-hide products to the existing costume-needle vambrace choices.
use super::{Definitions, Symbols};
use gielinor_world::tool_crafting::Recipe;
pub struct Group {
    pub recipes: Vec<Recipe>,
    pub costs: Vec<u16>,
    pub cycle_ticks: u8,
}
const ROWS: [(
    i32,
    bool,
    &str,
    i32,
    &str,
    u8,
    u16,
    &str,
    i32,
    &str,
    u8,
    u16,
); 4] = [
    (
        1745,
        false,
        "dragonhide_chaps",
        1099,
        "Green d'hide chaps",
        60,
        1240,
        "dragonhide_body",
        1135,
        "Green d'hide body",
        63,
        1860,
    ),
    (
        2505,
        true,
        "blue_dragonhide_chaps",
        2493,
        "Blue d'hide chaps",
        68,
        1400,
        "blue_dragonhide_body",
        2499,
        "Blue d'hide body",
        71,
        2100,
    ),
    (
        2507,
        true,
        "red_dragonhide_chaps",
        2495,
        "Red d'hide chaps",
        75,
        1560,
        "red_dragonhide_body",
        2501,
        "Red d'hide body",
        77,
        2340,
    ),
    (
        2509,
        true,
        "black_dragonhide_chaps",
        2497,
        "Black d'hide chaps",
        82,
        1720,
        "black_dragonhide_body",
        2503,
        "Black d'hide body",
        84,
        2580,
    ),
];
pub fn costume_needle_dragonhide(
    objects: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<Group>, String> {
    let base = super::costume_needle_leather(objects, sequences, stats, defs)?;
    let mut groups = Vec::new();
    for (
        raw,
        member,
        chaps_symbol,
        chaps,
        chaps_name,
        chaps_level,
        chaps_xp,
        body_symbol,
        body,
        body_name,
        body_level,
        body_xp,
    ) in ROWS
    {
        let (mut recipes, cycle_ticks) = base
            .iter()
            .find(|(rs, _)| rs[0].exchange.input == raw)
            .cloned()
            .ok_or("Missing original dragon vambrace dependency")?;
        for (symbol, id, name, level, xp) in [
            (chaps_symbol, chaps, chaps_name, chaps_level, chaps_xp),
            (body_symbol, body, body_name, body_level, body_xp),
        ] {
            let d = defs
                .items
                .get(&(id as u32))
                .ok_or("Missing dragonhide product")?;
            if objects.get(symbol) != Some(&id)
                || d.name != name
                || d.members != member
                || d.stackable
                || d.dummy == 1
                || d.cert.template != 0
                || d.placeholder.template != 0
                || d.transform.template != 0
                || d.inventory_ops[1].as_deref() != Some("Wear")
            {
                return Err(format!("Invalid dragonhide product {id}"));
            }
            let mut recipe = recipes[0];
            recipe.exchange.output = id;
            recipe.level = level;
            recipe.xp_tenths = xp;
            recipe.name = name;
            recipes.push(recipe);
        }
        groups.push(Group {
            recipes,
            costs: vec![1, 2, 3],
            cycle_ticks,
        });
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
        let (mut d, mut o, s, t) = super::super::costume_needle_leather::tests::profile();
        for (_, member, cs, c, cn, _, _, bs, b, bn, _, _) in ROWS {
            for (symbol, id, name) in [(cs, c, cn), (bs, b, bn)] {
                let mut item = gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();
                item.name = name.into();
                item.members = member;
                item.inventory_ops[1] = Some("Wear".into());
                d.items.insert(id as u32, item);
                o.insert(symbol.into(), id);
            }
        }
        (d, o, s, t)
    }
    #[test]
    fn costume_needle_dragonhide_resolves_four_complete_choice_groups_and_late_refusal() {
        let (mut d, o, s, t) = profile();
        let groups = costume_needle_dragonhide(&o, &s, &t, &d).unwrap();
        assert_eq!(groups.len(), 4);
        for g in groups {
            assert_eq!(g.recipes.len(), 3);
            assert_eq!(g.costs, vec![1, 2, 3]);
            assert_eq!(g.cycle_ticks, 3);
        }
        d.items.get_mut(&2503).unwrap().members = false;
        assert!(costume_needle_dragonhide(&o, &s, &t, &d).is_err());
    }
    #[test]
    fn costume_needle_dragonhide_actual_full_bag_grouped_hides_all_twelve_choices_both_orders() {
        let (d, o, s, t) = profile();
        for group in costume_needle_dragonhide(&o, &s, &t, &d).unwrap() {
            let recipes = std::sync::Arc::new(group.recipes);
            let costs = std::sync::Arc::new(group.costs);
            for index in 0..3 {
                for reverse in [false, true] {
                    let recipe = recipes[index];
                    let input = recipe.exchange.input;
                    let mut w = World::default();
                    let mut stats = Stats::default();
                    stats.set_base(12, 1);
                    stats.set_current(12, 84);
                    let mut data =
                        PlayerData::new(1, "Dragon maker".into(), CoordGrid::ZERO, stats);
                    data.xp_rate_hundreds = 125;
                    for slot in 0..28 {
                        data.inventory
                            .restore_slot(
                                slot,
                                Some(Item {
                                    id: input,
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
                    for id in [29920, input] {
                        w.held_use_types.insert(id, Type { id, content: -1 });
                    }
                    w.held_use_handlers
                        .register(
                            Scope::Types,
                            29920,
                            input,
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
                            selected_id: input,
                            target_slot: 4,
                            target_id: 29920,
                        }
                    } else {
                        Request {
                            selected_slot: 4,
                            selected_id: 29920,
                            target_slot: 17,
                            target_id: input,
                        }
                    };
                    w.finish_held_use(actor, held).unwrap();
                    let request = w.take_tool_crafting_request(actor).unwrap().unwrap();
                    assert_eq!(
                        request.available_for(index, &w.data(actor).unwrap().inventory),
                        27 / usize::from(costs[index])
                    );
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
                    assert_eq!(commits, vec![start + 2, start + 5]);
                    assert_eq!(old, 2);
                    assert_eq!(
                        data.inventory
                            .slots()
                            .iter()
                            .flatten()
                            .filter(|i| i.id == input)
                            .count(),
                        27 - 2 * usize::from(costs[index])
                    );
                    assert_eq!(
                        data.inventory.free_space(),
                        2 * (usize::from(costs[index]) - 1)
                    );
                    assert_eq!(data.inventory.get(4).unwrap().id, 29920);
                    assert_eq!(
                        data.stats.fine_xp(12),
                        (i32::from(recipe.xp_tenths) * 125 / 100) * 2
                    );
                }
            }
        }
    }
}
