//! Item manufacture only; placing,seed filling,trapping and retrieval are separate.
use super::{Definitions, Symbols};
use gielinor_world::assembly::{PreciseRecipe, Recipe, ToolRequirement};
const IMCANDO: ToolRequirement = ToolRequirement {
    inventory: &[25644, 29775],
    worn: &[25644, 29775],
};
const TOOLS: &[ToolRequirement] = &[
    ToolRequirement {
        inventory: &[2347, 25644, 29775],
        worn: &[25644, 29775],
    },
    ToolRequirement {
        inventory: &[1755],
        worn: &[],
    },
];
const ROWS: [(&str, i32, &str, bool, &str, i32, &str, u8, u16); 9] = [
    (
        "logs",
        1511,
        "Logs",
        false,
        "birdhouse_normal",
        21512,
        "Bird house",
        5,
        150,
    ),
    (
        "oak_logs",
        1521,
        "Oak logs",
        false,
        "birdhouse_oak",
        21515,
        "Oak bird house",
        15,
        200,
    ),
    (
        "willow_logs",
        1519,
        "Willow logs",
        false,
        "birdhouse_willow",
        21518,
        "Willow bird house",
        25,
        250,
    ),
    (
        "teak_logs",
        6333,
        "Teak logs",
        true,
        "birdhouse_teak",
        21521,
        "Teak bird house",
        35,
        300,
    ),
    (
        "maple_logs",
        1517,
        "Maple logs",
        false,
        "birdhouse_maple",
        22192,
        "Maple bird house",
        45,
        350,
    ),
    (
        "mahogany_logs",
        6332,
        "Mahogany logs",
        true,
        "birdhouse_mahogany",
        22195,
        "Mahogany bird house",
        50,
        400,
    ),
    (
        "yew_logs",
        1515,
        "Yew logs",
        false,
        "birdhouse_yew",
        22198,
        "Yew bird house",
        60,
        450,
    ),
    (
        "magic_logs",
        1513,
        "Magic logs",
        true,
        "birdhouse_magic",
        22201,
        "Magic bird house",
        75,
        500,
    ),
    (
        "redwood_logs",
        19669,
        "Redwood logs",
        true,
        "birdhouse_redwood",
        22204,
        "Redwood bird house",
        90,
        550,
    ),
];
pub fn birdhouses(
    objects: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<PreciseRecipe>, String> {
    if stats.get("crafting") != Some(&12)
        || sequences.get("birdhouse_make") != Some(&7057)
        || sequences.get("birdhouse_make_imcando_hammer") != Some(&8916)
    {
        return Err("Missing/aliased birdhouse skill/animations".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    let check = |symbol: &str, id: i32, name: &str, member: bool| -> Result<(), String> {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased birdhouse item {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing birdhouse item")?;
        if d.name != name
            || d.members != member
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid birdhouse item {id}"));
        }
        Ok(())
    };
    for (s, id, n, m) in [
        ("poh_clockwork_mechanism", 8792, "Clockwork", true),
        ("hammer", 2347, "Hammer", false),
        ("chisel", 1755, "Chisel", false),
        ("imcando_hammer", 25644, "Imcando hammer", true),
        (
            "imcando_hammer_offhand",
            29775,
            "Imcando hammer (off-hand)",
            true,
        ),
    ] {
        check(s, id, n, m)?;
    }
    let mut profiles = Vec::new();
    for (s, log, n, m, bs, out, bn, level, xp) in ROWS {
        check(s, log, n, m)?;
        check(bs, out, bn, true)?;
        profiles.push(PreciseRecipe {
            recipe: Recipe {
                first: log,
                second: 8792,
                output: out,
                skill: 12,
                level,
                base_xp: 0,
                members: true,
            },
            xp_tenths: xp,
            ticks: 2,
            animation: Some(7057),
            spot: None,
            required_tools: TOOLS,
            tool_animation: Some((IMCANDO, 8916)),
        });
    }
    Ok(profiles)
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        assembly::{self, Job, Outcome},
        inventory::{Inventory, Item, StackType},
        stats::Stats,
    };
    fn fixture() -> (Definitions, Symbols, Symbols, Symbols) {
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        defs.stats.insert(
            12,
            gielinor_cache::definitions::StatType::decode(b"\x01Crafting\0\0").unwrap(),
        );
        let mut obj = Symbols::new();
        let mut add = |s: &str, id: i32, n: &str, m: bool| {
            obj.insert(s.into(), id);
            let mut b = vec![2];
            b.extend_from_slice(n.as_bytes());
            b.push(0);
            if m {
                b.push(16);
            }
            b.push(0);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&b).unwrap(),
            );
        };
        for (s, id, n, m) in [
            ("poh_clockwork_mechanism", 8792, "Clockwork", true),
            ("hammer", 2347, "Hammer", false),
            ("chisel", 1755, "Chisel", false),
            ("imcando_hammer", 25644, "Imcando hammer", true),
            (
                "imcando_hammer_offhand",
                29775,
                "Imcando hammer (off-hand)",
                true,
            ),
        ] {
            add(s, id, n, m);
        }
        for (s, id, n, m, bs, out, bn, _, _) in ROWS {
            add(s, id, n, m);
            add(bs, out, bn, true);
        }
        (
            defs,
            obj,
            [
                ("birdhouse_make".into(), 7057),
                ("birdhouse_make_imcando_hammer".into(), 8916),
            ]
            .into(),
            [("crafting".into(), 12)].into(),
        )
    }
    #[test]
    fn birdhouses_all_nine_rewards_tools_and_imcando_inventory_worn_alternatives() {
        let (mut defs, obj, seq, stat) = fixture();
        let profiles = birdhouses(&obj, &seq, &stat, &defs).unwrap();
        assert_eq!(profiles.len(), 9);
        let cat = defs.inventory_catalog();
        for (r, fine) in profiles
            .into_iter()
            .zip([187, 250, 312, 375, 437, 500, 562, 625, 687])
        {
            for mode in 0..5 {
                let mut inv = Inventory::new(28, StackType::Normal, false);
                for slot in 0..28 {
                    inv.restore_slot(
                        slot,
                        Some(Item {
                            id: 1511,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                let mut worn = Inventory::new(14, StackType::Normal, false);
                for (slot, id) in [(1, 1755), (4, r.recipe.first), (19, 8792)] {
                    inv.restore_slot(
                        slot,
                        Some(Item {
                            id,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                if mode < 3 {
                    inv.restore_slot(
                        0,
                        Some(Item {
                            id: [2347, 25644, 29775][mode],
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                } else {
                    worn.restore_slot(
                        if mode == 3 { 3 } else { 5 },
                        Some(Item {
                            id: if mode == 3 { 25644 } else { 29775 },
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                assert!(assembly::tools_available(
                    r.required_tools,
                    &inv,
                    &worn,
                    &cat
                ));
                assert_eq!(
                    r.animation_for(&inv, &worn, &cat),
                    Some(if mode == 0 { 7057 } else { 8916 })
                );
                let mut stats = Stats::default();
                stats.set_current(12, r.recipe.level as i8);
                stats.set_current(21, 0);
                let before = (inv.clone(), worn.clone());
                let job = Job {
                    recipe: r.recipe,
                    first_slot: 4,
                    second_slot: 19,
                };
                let (out, _) = job.complete_awarding_tenths(
                    &mut inv,
                    &cat,
                    &mut stats,
                    125,
                    true,
                    r.xp_tenths,
                );
                assert_eq!(out, Outcome::Made);
                assert_eq!(stats.fine_xp(12), fine);
                assert_eq!(
                    inv.slots()
                        .iter()
                        .flatten()
                        .filter(|i| i.id == r.recipe.output)
                        .count(),
                    1
                );
                assert_eq!(worn, before.1);
                assert_eq!(inv.get(1), before.0.get(1));
                assert_eq!(inv.get(0), before.0.get(0));
                inv.restore_slot(1, None).unwrap();
                assert!(!assembly::tools_available(
                    r.required_tools,
                    &inv,
                    &worn,
                    &cat
                ));
            }
        }
        defs.items.get_mut(&29775).unwrap().members = false;
        assert!(birdhouses(&obj, &seq, &stat, &defs).is_err());
    }
    #[test]
    fn birdhouses_actual_quantity_two_then_three_ticks_and_late_tool_loss_preserves_prior_product()
    {
        use gielinor_world::{
            held_use::{Handler, Request, Scope, Type},
            simulation::{PlayerData, World, WorldEvent},
            CoordGrid,
        };
        let (defs, obj, seq, stat) = fixture();
        let profiles = birdhouses(&obj, &seq, &stat, &defs).unwrap();
        for index in [0, 3, 8] {
            for imcando in [false, true] {
                for lose_tool in [false, true] {
                    let r = profiles[index];
                    let mut w = World::default();
                    w.collision.allocate(3200, 3200, 0);
                    w.object_types = defs.inventory_catalog();
                    for id in [r.recipe.first, 8792] {
                        w.held_use_types.insert(id, Type { id, content: -1 });
                    }
                    w.held_use_handlers
                        .register(
                            Scope::Types,
                            r.recipe.first,
                            8792,
                            Handler::AssemblyProduction {
                                recipe: r,
                                cycle_ticks: 3,
                            },
                        )
                        .unwrap();
                    let mut stats = Stats::default();
                    stats.set_base(12, 90);
                    stats.set_current(12, 90);
                    stats.set_current(21, 0);
                    let mut data = PlayerData::new(
                        1,
                        "Birdhouse maker".into(),
                        CoordGrid::new(3200, 3200, 0).unwrap(),
                        stats,
                    );
                    data.xp_rate_hundreds = 125;
                    for slot in 0..28 {
                        data.inventory
                            .restore_slot(
                                slot,
                                Some(Item {
                                    id: 9951,
                                    count: 1,
                                    vars: 0,
                                }),
                            )
                            .unwrap();
                    }
                    data.inventory
                        .restore_slot(
                            1,
                            Some(Item {
                                id: 1755,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    if imcando {
                        data.equipment
                            .restore_slot(
                                5,
                                Some(Item {
                                    id: 29775,
                                    count: 1,
                                    vars: 0,
                                }),
                            )
                            .unwrap();
                    } else {
                        data.inventory
                            .restore_slot(
                                0,
                                Some(Item {
                                    id: 2347,
                                    count: 1,
                                    vars: 0,
                                }),
                            )
                            .unwrap();
                    }
                    // Non-material fillers must not become extra normal logs.
                    for slot in 0..28 {
                        if ![0, 1, 4, 8, 11, 17, 19, 22].contains(&slot) {
                            data.inventory
                                .restore_slot(
                                    slot,
                                    Some(Item {
                                        id: 9951,
                                        count: 1,
                                        vars: 0,
                                    }),
                                )
                                .unwrap();
                        }
                    }
                    for (slot, id) in [
                        (4, r.recipe.first),
                        (8, r.recipe.first),
                        (11, r.recipe.first),
                        (19, 8792),
                        (17, 8792),
                        (22, 8792),
                    ] {
                        data.inventory
                            .restore_slot(
                                slot,
                                Some(Item {
                                    id,
                                    count: 1,
                                    vars: 0,
                                }),
                            )
                            .unwrap();
                    }
                    w.queue_login(1, data).unwrap();
                    w.tick().unwrap();
                    let a = w.actors().next().unwrap();
                    w.set_membership(a, true).unwrap();
                    w.drain_events();
                    let held = if imcando {
                        Request {
                            selected_slot: 19,
                            selected_id: 8792,
                            target_slot: 4,
                            target_id: r.recipe.first,
                        }
                    } else {
                        Request {
                            selected_slot: 4,
                            selected_id: r.recipe.first,
                            target_slot: 19,
                            target_id: 8792,
                        }
                    };
                    if lose_tool {
                        w.data_mut(a)
                            .unwrap()
                            .inventory
                            .restore_slot(1, None)
                            .unwrap();
                        let inv = w.data(a).unwrap().inventory.clone();
                        w.prepare_held_use(a, held).unwrap();
                        w.finish_held_use(a, held).unwrap();
                        assert!(w.take_assembly_production_request(a).unwrap().is_none());
                        assert_eq!(w.data(a).unwrap().inventory, inv);
                        w.data_mut(a)
                            .unwrap()
                            .inventory
                            .restore_slot(
                                1,
                                Some(Item {
                                    id: 1755,
                                    count: 1,
                                    vars: 0,
                                }),
                            )
                            .unwrap();
                    }
                    w.prepare_held_use(a, held).unwrap();
                    w.finish_held_use(a, held).unwrap();
                    let request = w.take_assembly_production_request(a).unwrap().unwrap();
                    assert_eq!(request.available(&w.data(a).unwrap().inventory), 3);
                    w.start_assembly_production(a, request, 0, 3).unwrap();
                    let start = w.clock.0;
                    let mut commits = Vec::new();
                    let mut prior = 0;
                    for _ in 0..12 {
                        w.tick().unwrap();
                        let fine = w.data(a).unwrap().stats.fine_xp(12);
                        if fine != prior {
                            commits.push(w.clock.0 - 1);
                            prior = fine;
                            if lose_tool && commits.len() == 1 {
                                w.data_mut(a)
                                    .unwrap()
                                    .inventory
                                    .restore_slot(1, None)
                                    .unwrap();
                            }
                        }
                    }
                    assert_eq!(
                        commits,
                        if lose_tool {
                            vec![start + 2]
                        } else {
                            vec![start + 2, start + 5, start + 8]
                        }
                    );
                    let expected = if lose_tool { 1 } else { 3 };
                    let fine = [187, 250, 312, 375, 437, 500, 562, 625, 687][index];
                    assert_eq!(w.data(a).unwrap().stats.fine_xp(12), fine * expected);
                    assert_eq!(
                        w.data(a)
                            .unwrap()
                            .inventory
                            .slots()
                            .iter()
                            .flatten()
                            .filter(|i| i.id == r.recipe.output)
                            .count(),
                        expected as usize
                    );
                    let events = w.drain_events();
                    assert!(events.iter().any(|e|matches!(e,WorldEvent::PlayerCombatAnimation{sequence,..}if *sequence==if imcando{8916}else{7057})));
                    assert_eq!(
                        events
                            .iter()
                            .filter(|e| matches!(e, WorldEvent::SkillAdvanced { skill: 12, .. }))
                            .count(),
                        expected as usize
                    );
                    if lose_tool {
                        assert_eq!(
                            w.data(a).unwrap().inventory.get(8).unwrap().id,
                            r.recipe.first
                        );
                        assert_eq!(w.data(a).unwrap().inventory.get(17).unwrap().id, 8792);
                    }
                }
            }
        }
    }
}
