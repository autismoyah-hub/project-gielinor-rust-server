//! Hide shields share mixed material capture, stack debits and weak production.
use super::{Definitions, Symbols};
use gielinor_world::{grinding, held_use::Handler, tool_crafting::Recipe};
const ITEMS: [(&str, i32, &str, bool, bool); 27] = [
    ("hammer", 2347, "Hammer", false, false),
    ("imcando_hammer", 25644, "Imcando hammer", true, false),
    (
        "imcando_hammer_offhand",
        29775,
        "Imcando hammer (off-hand)",
        true,
        false,
    ),
    ("hard_leather", 1743, "Hard leather", false, false),
    ("oak_shield", 22251, "Oak shield", true, false),
    ("nails_bronze", 4819, "Bronze nails", true, true),
    ("leather_shield", 22269, "Hard leather shield", true, false),
    ("village_snake_skin", 6289, "Snakeskin", true, false),
    ("willow_shield", 22254, "Willow shield", true, false),
    ("nails_iron", 4820, "Iron nails", true, true),
    ("snakeskin_shield", 22272, "Snakeskin shield", true, false),
    ("dragon_leather", 1745, "Green dragon leather", true, false),
    ("maple_shield", 22257, "Maple shield", true, false),
    ("nails", 1539, "Steel nails", false, true),
    (
        "green_dhide_shield",
        22275,
        "Green d'hide shield",
        true,
        false,
    ),
    (
        "dragon_leather_blue",
        2505,
        "Blue dragon leather",
        true,
        false,
    ),
    ("yew_shield", 22260, "Yew shield", true, false),
    ("nails_mithril", 4822, "Mithril nails", true, true),
    (
        "blue_dhide_shield",
        22278,
        "Blue d'hide shield",
        true,
        false,
    ),
    (
        "dragon_leather_red",
        2507,
        "Red dragon leather",
        true,
        false,
    ),
    ("magic_shield", 22263, "Magic shield", true, false),
    ("nails_adamant", 4823, "Adamantite nails", true, true),
    ("red_dhide_shield", 22281, "Red d'hide shield", true, false),
    (
        "dragon_leather_black",
        2509,
        "Black dragon leather",
        true,
        false,
    ),
    ("redwood_shield", 22266, "Redwood shield", true, false),
    ("nails_rune", 4824, "Rune nails", true, true),
    (
        "black_dhide_shield",
        22284,
        "Black d'hide shield",
        true,
        false,
    ),
];
const ROWS: [(i32, i32, i32, i32, &str, u8, u16, &str, u16); 6] = [
    (
        1743,
        22251,
        4819,
        22269,
        "Hard leather shield",
        41,
        700,
        "human_shield_crafting_leather",
        7386,
    ),
    (
        6289,
        22254,
        4820,
        22272,
        "Snakeskin shield",
        56,
        1000,
        "human_shield_crafting_snakeskin",
        7387,
    ),
    (
        1745,
        22257,
        1539,
        22275,
        "Green d'hide shield",
        62,
        1240,
        "human_shield_crafting_green_dhide",
        7831,
    ),
    (
        2505,
        22260,
        4822,
        22278,
        "Blue d'hide shield",
        69,
        1400,
        "human_shield_crafting_blue_dhide",
        7832,
    ),
    (
        2507,
        22263,
        4823,
        22281,
        "Red d'hide shield",
        76,
        1560,
        "human_shield_crafting_red_dhide",
        7833,
    ),
    (
        2509,
        22266,
        4824,
        22284,
        "Black d'hide shield",
        83,
        1720,
        "human_shield_crafting_black_dhide",
        7834,
    ),
];
pub struct Shield {
    pub recipe: Recipe,
    pub materials: Vec<(i32, u16)>,
}
pub fn shield_crafting(
    objects: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<Shield>, String> {
    if stats.get("crafting") != Some(&12) {
        return Err("Missing shield Crafting alias".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    for &(symbol, id, name, members, stackable) in &ITEMS {
        let d = defs.items.get(&(id as u32)).ok_or("Missing shield item")?;
        if objects.get(symbol) != Some(&id)
            || d.name != name
            || d.members != members
            || d.stackable != stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
            || (id == 25644 && d.wear != [3, -1, -1])
            || (id == 29775 && d.wear != [5, -1, -1])
            || (ROWS.iter().any(|v| v.3 == id) && d.inventory_ops[1].as_deref() != Some("Wield"))
        {
            return Err(format!("Invalid shield material/output {id}"));
        }
    }
    let mut recipes = Vec::new();
    for &(raw, shield, nails, output, name, level, xp_tenths, sequence, animation) in &ROWS {
        if sequences.get(sequence) != Some(&i32::from(animation)) {
            return Err("Missing shield animation alias".into());
        }
        recipes.push(Shield {
            recipe: Recipe {
                skill: 12,
                exchange: grinding::Recipe {
                    tool: 2347,
                    input: raw,
                    output,
                    members: true,
                    animation: Some(animation),
                },
                level,
                xp_tenths,
                output_count: 1,
                ticks: 5,
                name,
                action: "make",
                failure: None,
            },
            materials: vec![(raw, 2), (shield, 1), (nails, 15)],
        });
    }
    Ok(recipes)
}
pub fn shield_crafting_registrations(profiles: &[Shield]) -> Vec<(i32, i32, Handler)> {
    let mut entries = Vec::new();
    for p in profiles {
        let materials = [p.materials[0].0, p.materials[1].0, p.materials[2].0];
        let pairs =
            [2347, 25644, 29775]
                .into_iter()
                .flat_map(|tool| materials.into_iter().map(move |raw| (tool, raw)))
                .chain((0..materials.len()).flat_map(|i| {
                    (i + 1..materials.len()).map(move |j| (materials[i], materials[j]))
                }));
        for (a, b) in pairs {
            entries.push((
                a,
                b,
                Handler::ToolCraftingEquivalentMixedChoices {
                    recipes: std::sync::Arc::new(vec![p.recipe]),
                    materials: std::sync::Arc::new(vec![p.materials.clone()]),
                    cycle_ticks: 5,
                    tools: std::sync::Arc::new(vec![2347, 25644, 29775]),
                    worn_tools: std::sync::Arc::new(vec![(25644, 3), (29775, 5)]),
                },
            ));
        }
    }
    entries
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        held_use::{Request, Scope, Type},
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
        for &(symbol, id, name, members, stackable) in &ITEMS {
            let mut item = gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();
            item.name = name.into();
            item.members = members;
            item.stackable = stackable;
            if id == 25644 {
                item.wear = [3, -1, -1];
            }
            if id == 29775 {
                item.wear = [5, -1, -1];
            }
            if ROWS.iter().any(|v| v.3 == id) {
                item.inventory_ops[1] = Some("Wield".into());
            }
            d.items.insert(id as u32, item);
            o.insert(symbol.into(), id);
        }
        d.stats.insert(
            12,
            gielinor_cache::definitions::StatType::decode(b"\x01Crafting\0\0").unwrap(),
        );
        (
            d,
            o,
            ROWS.iter().map(|v| (v.7.into(), i32::from(v.8))).collect(),
            [("crafting".into(), 12)].into(),
        )
    }
    #[test]
    fn shield_crafting_exact_profiles_six_pairs_each_and_late_invalid_nails() {
        let (mut d, o, s, t) = profile();
        let ps = shield_crafting(&o, &s, &t, &d).unwrap();
        assert_eq!(ps.len(), 6);
        let entries = shield_crafting_registrations(&ps);
        assert_eq!(entries.len(), 72);
        let pairs = entries
            .iter()
            .map(|v| if v.0 < v.1 { (v.0, v.1) } else { (v.1, v.0) })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(pairs.len(), 72);
        assert!(ps.iter().all(|p| p.recipe.ticks == 5
            && p.materials.iter().map(|v| v.1).collect::<Vec<_>>() == vec![2, 1, 15]));
        d.items.get_mut(&4824).unwrap().stackable = false;
        assert!(shield_crafting(&o, &s, &t, &d).is_err());
    }
    fn fixture(
        index: usize,
    ) -> (
        World,
        gielinor_world::simulation::ActorId,
        Recipe,
        Vec<(i32, u16)>,
    ) {
        let (d, o, s, t) = profile();
        let ps = shield_crafting(&o, &s, &t, &d).unwrap();
        let p = &ps[index];
        let mut w = World::default();
        let mut stats = Stats::default();
        stats.set_base(12, 1);
        stats.set_current(12, 83);
        let mut data = PlayerData::new(1, "Shield maker".into(), CoordGrid::ZERO, stats);
        data.xp_rate_hundreds = 125;
        for slot in 0..28 {
            data.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id: if slot < 18 {
                            p.materials[0].0
                        } else {
                            p.materials[1].0
                        },
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
                    id: 2347,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        data.inventory
            .restore_slot(
                23,
                Some(Item {
                    id: p.materials[2].0,
                    count: 31,
                    vars: 0,
                }),
            )
            .unwrap();
        w.object_types.stackable.insert(p.materials[2].0);
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.set_membership(a, true).unwrap();
        for id in [2347, p.materials[0].0, p.materials[1].0, p.materials[2].0] {
            w.held_use_types.insert(id, Type { id, content: -1 });
        }
        for (a, b, h) in shield_crafting_registrations(&ps) {
            w.held_use_handlers.register(Scope::Types, a, b, h).unwrap();
        }
        (w, a, p.recipe, p.materials.clone())
    }
    fn held(materials: &[(i32, u16)], i: usize, j: usize, reverse: bool) -> Request {
        let pairs = [
            (4, 2347),
            (7, materials[0].0),
            (19, materials[1].0),
            (23, materials[2].0),
        ];
        let ((a, ai), (b, bi)) = (pairs[i], pairs[j]);
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
    #[test]
    fn shield_crafting_actual_all_six_twelve_dispatch_routes_stacked_nails_fullbag() {
        for index in 0..6 {
            for i in 0..4 {
                for j in i + 1..4 {
                    for reverse in [false, true] {
                        let (mut w, a, r, materials) = fixture(index);
                        w.finish_held_use(a, held(&materials, i, j, reverse))
                            .unwrap();
                        let req = w.take_tool_crafting_request(a).unwrap().unwrap();
                        assert_eq!(req.available_for(0, &w.data(a).unwrap().inventory), 2);
                        let start = w.clock.0;
                        w.start_tool_crafting_choice(a, req, 0, 28).unwrap();
                        let mut times = Vec::new();
                        let mut old = 0;
                        for _ in 0..13 {
                            w.tick().unwrap();
                            let n = w
                                .data(a)
                                .unwrap()
                                .inventory
                                .slots()
                                .iter()
                                .flatten()
                                .filter(|v| v.id == r.exchange.output)
                                .count();
                            if n != old {
                                times.push(w.clock.0 - 1);
                            }
                            old = n;
                        }
                        let data = w.data(a).unwrap();
                        let count = |id| {
                            data.inventory
                                .slots()
                                .iter()
                                .flatten()
                                .filter(|v| v.id == id)
                                .map(|v| v.count)
                                .sum::<i32>()
                        };
                        assert_eq!(times, vec![start + 5, start + 10]);
                        assert_eq!(old, 2);
                        assert_eq!(count(materials[0].0), 13);
                        assert_eq!(count(materials[1].0), 7);
                        assert_eq!(count(materials[2].0), 1);
                        assert_eq!(data.inventory.get(4).unwrap().id, 2347);
                        assert_eq!(data.inventory.free_space(), 4);
                        assert_eq!(
                            data.stats.fine_xp(12),
                            2 * (i32::from(r.xp_tenths) * 125 / 100)
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn shield_crafting_selected_nail_stack_and_late_depletion_fail_without_substitution() {
        for case in 0..3 {
            let (mut w, a, r, materials) = fixture(0);
            w.finish_held_use(a, held(&materials, 2, 3, false)).unwrap();
            let req = w.take_tool_crafting_request(a).unwrap().unwrap();
            if case == 0 {
                w.data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        23,
                        Some(Item {
                            id: materials[2].0,
                            count: 14,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                assert!(w.start_tool_crafting_choice(a, req, 0, 2).is_err());
                continue;
            }
            w.start_tool_crafting_choice(a, req, 0, 2).unwrap();
            for _ in 0..6 {
                w.tick().unwrap();
            }
            let data = w.data_mut(a).unwrap();
            data.inventory
                .restore_slot(
                    23,
                    Some(Item {
                        id: materials[2].0,
                        count: if case == 1 { 14 } else { 16 },
                        vars: if case == 2 { 9 } else { 0 },
                    }),
                )
                .unwrap();
            // A different stack must not replace the accepted stack.
            data.inventory
                .restore_slot(
                    10,
                    Some(Item {
                        id: materials[2].0,
                        count: 100,
                        vars: 0,
                    }),
                )
                .unwrap();
            let inv = data.inventory.clone();
            let xp = data.stats.fine_xp(12);
            for _ in 0..8 {
                w.tick().unwrap();
            }
            let data = w.data(a).unwrap();
            assert_eq!(data.inventory, inv);
            assert_eq!(data.stats.fine_xp(12), xp);
            assert_eq!(
                data.inventory
                    .slots()
                    .iter()
                    .flatten()
                    .filter(|v| v.id == r.exchange.output)
                    .count(),
                1
            );
        }
    }
    #[test]
    fn shield_crafting_held_imcando_both_forms_explicit_and_material_pairs_pin_the_tool() {
        for tool in [25644, 29775] {
            for (i, j) in [(0, 1), (0, 2), (0, 3), (2, 3)] {
                for reverse in [false, true] {
                    let (mut w, a, r, materials) = fixture(0);
                    w.data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(
                            4,
                            Some(Item {
                                id: tool,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    w.held_use_types.insert(
                        tool,
                        Type {
                            id: tool,
                            content: -1,
                        },
                    );
                    let mut request = held(&materials, i, j, reverse);
                    if request.selected_id == 2347 {
                        request.selected_id = tool;
                    }
                    if request.target_id == 2347 {
                        request.target_id = tool;
                    }
                    w.finish_held_use(a, request).unwrap();
                    let req = w.take_tool_crafting_request(a).unwrap().unwrap();
                    assert_eq!(req.recipes[0].exchange.tool, tool);
                    assert_eq!(req.tool_slot, 4);
                    w.start_tool_crafting_choice(a, req, 0, 1).unwrap();
                    for _ in 0..6 {
                        w.tick().unwrap();
                    }
                    let d = w.data(a).unwrap();
                    assert_eq!(d.inventory.get(4).unwrap().id, tool);
                    assert_eq!(d.inventory.get(23).unwrap().count, 16);
                    assert_eq!(
                        d.inventory
                            .slots()
                            .iter()
                            .flatten()
                            .filter(|v| v.id == r.exchange.output)
                            .count(),
                        1
                    );
                    assert_eq!(d.stats.fine_xp(12), 875);
                }
            }
        }
        // The clicked tool wins over another available equivalent; after acceptance no fallback.
        let (mut w, a, _, materials) = fixture(0);
        w.data_mut(a)
            .unwrap()
            .inventory
            .restore_slot(
                10,
                Some(Item {
                    id: 25644,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        w.held_use_types.insert(
            25644,
            Type {
                id: 25644,
                content: -1,
            },
        );
        w.finish_held_use(
            a,
            Request {
                selected_slot: 10,
                selected_id: 25644,
                target_slot: 19,
                target_id: materials[1].0,
            },
        )
        .unwrap();
        let req = w.take_tool_crafting_request(a).unwrap().unwrap();
        assert_eq!(req.tool_slot, 10);
        assert_eq!(req.recipes[0].exchange.tool, 25644);
        w.start_tool_crafting_choice(a, req, 0, 1).unwrap();
        w.data_mut(a)
            .unwrap()
            .inventory
            .restore_slot(10, None)
            .unwrap();
        let before = w.data(a).unwrap().inventory.clone();
        for _ in 0..7 {
            w.tick().unwrap();
        }
        assert_eq!(w.data(a).unwrap().inventory, before);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(12), 0);
    }
    #[test]
    fn shield_crafting_worn_imcando_both_hands_all_material_pairs_fullbag_distinct_slot_spaces() {
        for (tool, wear) in [(25644, 3), (29775, 5)] {
            for (i, j) in [(1, 2), (1, 3), (2, 3)] {
                for reverse in [false, true] {
                    let (mut w, a, r, materials) = fixture(0);
                    let data = w.data_mut(a).unwrap();
                    data.inventory
                        .restore_slot(
                            4,
                            Some(Item {
                                id: materials[0].0,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    data.equipment
                        .restore_slot(
                            wear,
                            Some(Item {
                                id: tool,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    let gear = data.equipment.clone();
                    w.finish_held_use(a, held(&materials, i, j, reverse))
                        .unwrap();
                    let req = w.take_tool_crafting_request(a).unwrap().unwrap();
                    assert_eq!(req.worn_tool_slot, Some(wear));
                    assert_eq!(req.tool_slot, usize::MAX);
                    assert_eq!(req.recipes[0].exchange.tool, tool);
                    w.start_tool_crafting_choice(a, req, 0, 2).unwrap();
                    for _ in 0..12 {
                        w.tick().unwrap();
                    }
                    let data = w.data(a).unwrap();
                    assert_eq!(data.equipment, gear);
                    assert_eq!(
                        data.inventory
                            .slots()
                            .iter()
                            .flatten()
                            .filter(|v| v.id == r.exchange.output)
                            .count(),
                        2
                    );
                    assert_eq!(
                        data.inventory
                            .slots()
                            .iter()
                            .flatten()
                            .filter(|v| v.id == materials[0].0)
                            .count(),
                        14
                    );
                    assert_eq!(data.inventory.get(23).unwrap().count, 1);
                    assert_eq!(data.inventory.free_space(), 4);
                    assert_eq!(data.stats.fine_xp(12), 1750);
                    if i == 2 && j == 3 {
                        assert!(data.inventory.get(3).is_none());
                    }
                }
            }
        }
    }
    #[test]
    fn shield_crafting_worn_tool_late_removal_variables_wrong_hand_and_membership_never_substitute()
    {
        for case in 0..4 {
            let (mut w, a, r, materials) = fixture(0);
            let data = w.data_mut(a).unwrap();
            data.inventory
                .restore_slot(
                    4,
                    Some(Item {
                        id: materials[0].0,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            data.equipment
                .restore_slot(
                    3,
                    Some(Item {
                        id: 25644,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            w.finish_held_use(a, held(&materials, 2, 3, false)).unwrap();
            let req = w.take_tool_crafting_request(a).unwrap().unwrap();
            w.start_tool_crafting_choice(a, req, 0, 2).unwrap();
            for _ in 0..6 {
                w.tick().unwrap();
            }
            let data = w.data_mut(a).unwrap();
            match case {
                0 => {
                    data.equipment.restore_slot(3, None).unwrap();
                }
                1 => {
                    data.equipment
                        .restore_slot(
                            3,
                            Some(Item {
                                id: 25644,
                                count: 1,
                                vars: 9,
                            }),
                        )
                        .unwrap();
                }
                2 => {
                    data.equipment.restore_slot(3, None).unwrap();
                    data.equipment
                        .restore_slot(
                            5,
                            Some(Item {
                                id: 25644,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                _ => {}
            }
            data.inventory
                .restore_slot(
                    10,
                    Some(Item {
                        id: 2347,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            let inv = data.inventory.clone();
            let gear = data.equipment.clone();
            let xp = data.stats.fine_xp(12);
            if case == 3 {
                w.set_membership(a, false).unwrap();
            }
            for _ in 0..8 {
                w.tick().unwrap();
            }
            let data = w.data(a).unwrap();
            assert_eq!(data.inventory, inv);
            assert_eq!(data.equipment, gear);
            assert_eq!(data.stats.fine_xp(12), xp);
            assert_eq!(
                data.inventory
                    .slots()
                    .iter()
                    .flatten()
                    .filter(|v| v.id == r.exchange.output)
                    .count(),
                1
            );
        }
    }
}
