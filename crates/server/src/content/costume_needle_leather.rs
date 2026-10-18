//! Basic leather needlework through the source's thread-free costume needle.
use super::{Definitions, Symbols};
use gielinor_world::{grinding, tool_crafting::Recipe};
const SOFT: [(&str, i32, &str, u8, u16); 7] = [
    ("leather_gloves", 1059, "Leather gloves", 1, 138),
    ("leather_boots", 1061, "Leather boots", 7, 163),
    ("leather_cowl", 1167, "Leather cowl", 9, 185),
    ("leather_vambraces", 1063, "Leather vambraces", 11, 220),
    ("leather_armour", 1129, "Leather body", 14, 250),
    ("leather_chaps", 1095, "Leather chaps", 18, 270),
    ("coif", 1169, "Coif", 38, 370),
];
const DRAGON: [(&str, i32, &str, &str, i32, &str, bool, u8, u16); 4] = [
    (
        "dragon_leather",
        1745,
        "Green dragon leather",
        "dragon_vambraces",
        1065,
        "Green d'hide vambraces",
        false,
        57,
        620,
    ),
    (
        "dragon_leather_blue",
        2505,
        "Blue dragon leather",
        "blue_dragon_vambraces",
        2487,
        "Blue d'hide vambraces",
        true,
        66,
        700,
    ),
    (
        "dragon_leather_red",
        2507,
        "Red dragon leather",
        "red_dragon_vambraces",
        2489,
        "Red d'hide vambraces",
        true,
        73,
        780,
    ),
    (
        "dragon_leather_black",
        2509,
        "Black dragon leather",
        "black_dragon_vambraces",
        2491,
        "Black d'hide vambraces",
        true,
        79,
        860,
    ),
];
pub fn costume_needle_leather(
    objects: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<(Vec<Recipe>, u8)>, String> {
    if stats.get("crafting") != Some(&12) || sequences.get("human_leather_crafting") != Some(&1249)
    {
        return Err("Missing leather Crafting/animation alias".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    let check =
        |symbol: &str, id: i32, name: &str, member: bool, wear: bool| -> Result<(), String> {
            let d = defs
                .items
                .get(&(id as u32))
                .ok_or("Missing needlework item")?;
            if objects.get(symbol) != Some(&id)
                || d.name != name
                || d.members != member
                || d.stackable
                || d.dummy == 1
                || d.cert.template != 0
                || d.placeholder.template != 0
                || d.transform.template != 0
                || (wear && d.inventory_ops[1].as_deref() != Some("Wear"))
            {
                return Err(format!("Invalid needlework item {id}"));
            }
            Ok(())
        };
    check("costumeneedle", 29920, "Costume needle", true, false)?;
    check("leather", 1741, "Leather", false, false)?;
    check("hard_leather", 1743, "Hard leather", false, false)?;
    let make = |input, output, level, xp_tenths, name| Recipe {
        skill: 12,
        exchange: grinding::Recipe {
            tool: 29920,
            input,
            output,
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
    };
    let mut soft = Vec::new();
    for (symbol, id, name, level, xp) in SOFT {
        check(symbol, id, name, false, true)?;
        soft.push(make(1741, id, level, xp, name));
    }
    check("hardleather_body", 1131, "Hardleather body", false, true)?;
    // Source Needlework uses3 globally; pinned Wiki Hardleather body specifies2.
    let mut groups = vec![
        (soft, 3),
        (vec![make(1743, 1131, 28, 350, "Hardleather body")], 2),
    ];
    for (raw_symbol, raw, raw_name, out_symbol, out, name, member, level, xp) in DRAGON {
        check(raw_symbol, raw, raw_name, true, false)?;
        check(out_symbol, out, name, member, true)?;
        groups.push((vec![make(raw, out, level, xp, name)], 3));
    }
    Ok(groups)
}
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use gielinor_world::{
        held_use::{Handler, Request, Scope, Type},
        inventory::Item,
        simulation::{PlayerData, World},
        stats::Stats,
        CoordGrid,
    };
    pub(crate) fn profile() -> (Definitions, Symbols, Symbols, Symbols) {
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
        let mut all = vec![
            ("costumeneedle", 29920, "Costume needle", true, false),
            ("leather", 1741, "Leather", false, false),
            ("hard_leather", 1743, "Hard leather", false, false),
            ("hardleather_body", 1131, "Hardleather body", false, true),
        ];
        all.extend(SOFT.map(|(s, id, n, _, _)| (s, id, n, false, true)));
        for (rs, raw, rn, os, out, on, member, _, _) in DRAGON {
            all.push((rs, raw, rn, true, false));
            all.push((os, out, on, member, true));
        }
        for (s, id, n, m, wear) in all {
            let mut item = gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();
            item.name = n.into();
            item.members = m;
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
    fn costume_needle_leather_exact_twelve_member_routes_and_wiki_hardleather_cycle() {
        let (mut d, o, s, t) = profile();
        let groups = costume_needle_leather(&o, &s, &t, &d).unwrap();
        assert_eq!(groups[0].0.len(), 7);
        assert_eq!(groups.iter().map(|g| g.0.len()).sum::<usize>(), 12);
        assert_eq!(groups[0].1, 3);
        assert_eq!(groups[1].1, 2);
        assert!(groups
            .iter()
            .flat_map(|g| g.0.iter())
            .all(|r| r.exchange.members && r.ticks == 2));
        d.items.get_mut(&1131).unwrap().cert.template = 799;
        assert!(costume_needle_leather(&o, &s, &t, &d).is_err());
    }
    #[test]
    fn costume_needle_leather_actual_full_bag_both_orders_fractional_xp_and_two_input_cycles() {
        let (d, o, s, t) = profile();
        for (recipes, cycle) in costume_needle_leather(&o, &s, &t, &d).unwrap() {
            let recipes = std::sync::Arc::new(recipes);
            for index in 0..recipes.len() {
                for reverse in [false, true] {
                    let mut w = World::default();
                    let mut stats = Stats::default();
                    stats.set_base(12, 1);
                    stats.set_current(12, 79);
                    let mut data =
                        PlayerData::new(1, "Leather maker".into(), CoordGrid::ZERO, stats);
                    data.xp_rate_hundreds = 125;
                    for slot in 0..28 {
                        data.inventory
                            .restore_slot(
                                slot,
                                Some(Item {
                                    id: 1511,
                                    count: 1,
                                    vars: 0,
                                }),
                            )
                            .unwrap();
                    }
                    let input = recipes[index].exchange.input;
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
                    for slot in [17, 23] {
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
                            Handler::ToolCraftingChoices {
                                recipes: recipes.clone(),
                                cycle_ticks: cycle,
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
                    assert_eq!(request.input_count, 1);
                    let start = w.clock.0;
                    w.start_tool_crafting_choice(actor, request, index, 2)
                        .unwrap();
                    let mut commits = Vec::new();
                    let output = recipes[index].exchange.output;
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
                            .filter(|i| i.id == output)
                            .count();
                        if now != old {
                            commits.push(w.clock.0 - 1);
                        }
                        old = now;
                    }
                    let data = w.data(actor).unwrap();
                    assert_eq!(commits, vec![start + 2, start + 2 + i32::from(cycle)]);
                    assert_eq!(old, 2);
                    assert_eq!(data.inventory.free_space(), 0);
                    assert_eq!(data.inventory.get(17).unwrap().id, output);
                    assert_eq!(data.inventory.get(23).unwrap().id, output);
                    assert_eq!(data.inventory.get(4).unwrap().id, 29920);
                    assert_eq!(
                        data.stats.fine_xp(12),
                        (i32::from(recipes[index].xp_tenths) * 125 / 100) * 2
                    );
                }
            }
        }
    }
}
