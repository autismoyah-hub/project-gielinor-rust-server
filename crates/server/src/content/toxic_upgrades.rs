//! Uncharged magic-fang assembly; charging, combat and dismantling are separate.
use super::{Definitions, Symbols};
use gielinor_world::assembly::{PreciseRecipe, Recipe, ToolRequirement};
const TOOLS: &[ToolRequirement] = &[ToolRequirement {
    inventory: &[1755],
    worn: &[],
}];
const ITEMS: [(&str, i32, &str, bool); 6] = [
    ("magic_fang", 12932, "Magic fang", true),
    ("sotd", 11791, "Staff of the dead", true),
    ("toxic_sotd", 12902, "Toxic staff (uncharged)", true),
    ("tots_uncharged", 11908, "Uncharged trident", true),
    (
        "toxic_tots_uncharged",
        12900,
        "Uncharged toxic trident",
        true,
    ),
    ("chisel", 1755, "Chisel", false),
];
pub fn toxic_upgrades(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<PreciseRecipe>, String> {
    if stats.get("crafting") != Some(&12) {
        return Err("Missing/aliased Crafting stat".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    for (symbol, id, name, members) in ITEMS {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased toxic upgrade item {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing toxic upgrade item")?;
        if d.name != name
            || d.members != members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid toxic upgrade item {id}"));
        }
    }
    Ok([(11791, 12902), (11908, 12900)]
        .into_iter()
        .map(|(second, output)| PreciseRecipe {
            recipe: Recipe {
                first: 12932,
                second,
                output,
                skill: 12,
                level: 59,
                base_xp: 0,
                members: true,
            },
            xp_tenths: 0,
            ticks: 1,
            animation: None,
            spot: None,
            required_tools: TOOLS,
            tool_animation: None,
        })
        .collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        held_use::{Handler, Request, Scope, Type},
        inventory::{Inventory, Item},
        simulation::{PlayerData, World, WorldEvent},
        stats::Stats,
        CoordGrid,
    };
    fn fixture() -> (Definitions, Symbols, Symbols) {
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
        let mut objects = Symbols::new();
        for (symbol, id, name, members) in ITEMS {
            objects.insert(symbol.into(), id);
            let mut b = vec![2];
            b.extend_from_slice(name.as_bytes());
            b.push(0);
            if members {
                b.push(16);
            }
            b.push(0);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&b).unwrap(),
            );
        }
        (defs, objects, [("crafting".into(), 12)].into())
    }
    #[test]
    fn toxic_upgrades_validate_definitions_and_durable_tool() {
        let (mut defs, objects, names) = fixture();
        let rows = toxic_upgrades(&objects, &names, &defs).unwrap();
        assert_eq!(rows.len(), 2);
        let inv = Inventory::new(28, gielinor_world::inventory::StackType::Normal, false);
        let mut held = inv.clone();
        let worn = inv.clone();
        let cat = defs.inventory_catalog();
        assert!(!gielinor_world::assembly::tools_available(
            rows[0].required_tools,
            &held,
            &worn,
            &cat
        ));
        held.restore_slot(
            3,
            Some(Item {
                id: 1755,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        assert!(gielinor_world::assembly::tools_available(
            rows[0].required_tools,
            &held,
            &worn,
            &cat
        ));
        defs.items.get_mut(&12900).unwrap().cert.template = 799;
        assert!(toxic_upgrades(&objects, &names, &defs).is_err());
    }
    #[test]
    fn toxic_upgrades_actual_pair_both_orders_full_bag_and_live_refusals() {
        let (defs, objects, names) = fixture();
        for r in toxic_upgrades(&objects, &names, &defs).unwrap() {
            for reverse in [false, true] {
                for refusal in 0..7 {
                    let mut w = World::default();
                    w.collision.allocate(3200, 3200, 0);
                    w.object_types = defs.inventory_catalog();
                    for id in [r.recipe.first, r.recipe.second] {
                        w.held_use_types.insert(id, Type { id, content: -1 });
                    }
                    w.held_use_handlers
                        .register(
                            Scope::Types,
                            r.recipe.first,
                            r.recipe.second,
                            Handler::AssemblyPrecise(r),
                        )
                        .unwrap();
                    let mut stats = Stats::default();
                    stats.set_base(12, 1);
                    stats.set_current(12, 59);
                    stats.set_current(6, 0);
                    stats.set_current(0, 0);
                    stats.set_fine_xp(12, 12345).unwrap();
                    let mut data = PlayerData::new(
                        1,
                        "Fang assembler".into(),
                        CoordGrid::new(3200, 3200, 0).unwrap(),
                        stats,
                    );
                    data.xp_rate_hundreds = 0;
                    for slot in 0..28 {
                        data.inventory
                            .restore_slot(
                                slot,
                                Some(Item {
                                    id: 1755,
                                    count: 1,
                                    vars: 0,
                                }),
                            )
                            .unwrap();
                    }
                    for (slot, id) in [
                        (4, r.recipe.first),
                        (19, r.recipe.second),
                        (8, r.recipe.second),
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
                    // One durable chisel only, so removing it actually revokes the requirement.
                    for slot in 0..28 {
                        if ![3, 4, 8, 19].contains(&slot) {
                            data.inventory
                                .restore_slot(
                                    slot,
                                    Some(Item {
                                        id: r.recipe.second,
                                        count: 1,
                                        vars: 0,
                                    }),
                                )
                                .unwrap();
                        }
                    }
                    w.queue_login(1, data).unwrap();
                    w.tick().unwrap();
                    let a = w.actors().next().unwrap();
                    w.set_membership(a, true).unwrap();
                    w.drain_events();
                    let request = if reverse {
                        Request {
                            selected_slot: 19,
                            selected_id: r.recipe.second,
                            target_slot: 4,
                            target_id: r.recipe.first,
                        }
                    } else {
                        Request {
                            selected_slot: 4,
                            selected_id: r.recipe.first,
                            target_slot: 19,
                            target_id: r.recipe.second,
                        }
                    };
                    w.prepare_held_use(a, request).unwrap();
                    w.finish_held_use(a, request).unwrap();
                    let start = w.clock.0;
                    match refusal {
                        1 => w.set_membership(a, false).unwrap(),
                        2 => {
                            w.data_mut(a).unwrap().stats.set_current(12, 58);
                        }
                        3 => {
                            w.object_types.dummy.insert(r.recipe.output);
                        }
                        4 => {
                            w.data_mut(a)
                                .unwrap()
                                .inventory
                                .restore_slot(
                                    19,
                                    Some(Item {
                                        id: r.recipe.second,
                                        count: 1,
                                        vars: 9,
                                    }),
                                )
                                .unwrap();
                        }
                        5 => {
                            w.data_mut(a)
                                .unwrap()
                                .inventory
                                .restore_slot(3, None)
                                .unwrap();
                        }
                        6 => {
                            w.data_mut(a)
                                .unwrap()
                                .inventory
                                .restore_slot(
                                    3,
                                    Some(Item {
                                        id: 1755,
                                        count: 1,
                                        vars: 9,
                                    }),
                                )
                                .unwrap();
                        }
                        _ => {}
                    }
                    let expected_inv = w.data(a).unwrap().inventory.clone();
                    let expected_stats = w.data(a).unwrap().stats.clone();
                    let mut commits = Vec::new();
                    let mut seen = false;
                    for _ in 0..5 {
                        w.tick().unwrap();
                        let made = w
                            .data(a)
                            .unwrap()
                            .inventory
                            .slots()
                            .iter()
                            .flatten()
                            .any(|i| i.id == r.recipe.output);
                        if made && !seen {
                            commits.push(w.clock.0 - 1);
                            seen = true;
                        }
                    }
                    assert_eq!(seen, refusal == 0);
                    if refusal == 0 {
                        assert_eq!(commits, vec![start + 1]);
                        assert_eq!(w.data(a).unwrap().stats, expected_stats);
                        assert_eq!(w.data(a).unwrap().inventory.get(3).unwrap().id, 1755);
                        assert_eq!(
                            w.data(a).unwrap().inventory.get(8).unwrap().id,
                            r.recipe.second
                        );
                        assert_eq!(w.data(a).unwrap().inventory.free_space(), 1);
                    } else {
                        assert_eq!(w.data(a).unwrap().inventory, expected_inv);
                        assert_eq!(w.data(a).unwrap().stats, expected_stats);
                    }
                    assert!(!w
                        .drain_events()
                        .iter()
                        .any(|e| matches!(e, WorldEvent::SkillAdvanced { .. })));
                }
            }
        }
    }
}
