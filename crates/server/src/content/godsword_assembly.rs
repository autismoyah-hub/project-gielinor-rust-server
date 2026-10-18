//! Completed blade plus hilt only; shard forging and dismantling are separate.
use super::{Definitions, Symbols};
use gielinor_world::assembly::Recipe;
pub fn godsword_attachments(objects: &Symbols, defs: &Definitions) -> Result<Vec<Recipe>, String> {
    let check = |symbol: &str, id: i32, name: &str| -> Result<(), String> {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased Godsword object {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing Godsword attachment definition")?;
        if d.name != name
            || !d.members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid Godsword attachment definition {id}"));
        }
        Ok(())
    };
    check("godwars_godsword_blade1+2+3", 11798, "Godsword blade")?;
    let mut recipes = Vec::new();
    for (hilt_symbol, hilt, hilt_name, out_symbol, out, name) in [
        (
            "godwars_godsword_hilt_armadyl",
            11810,
            "Armadyl hilt",
            "ags",
            11802,
            "Armadyl godsword",
        ),
        (
            "godwars_godsword_hilt_bandos",
            11812,
            "Bandos hilt",
            "bgs",
            11804,
            "Bandos godsword",
        ),
        (
            "godwars_godsword_hilt_saradomin",
            11814,
            "Saradomin hilt",
            "sgs",
            11806,
            "Saradomin godsword",
        ),
        (
            "godwars_godsword_hilt_zamorak",
            11816,
            "Zamorak hilt",
            "zgs",
            11808,
            "Zamorak godsword",
        ),
        (
            "godwars_godsword_hilt_ancient",
            26370,
            "Ancient hilt",
            "ancient_godsword",
            26233,
            "Ancient godsword",
        ),
    ] {
        check(hilt_symbol, hilt, hilt_name)?;
        check(out_symbol, out, name)?;
        recipes.push(Recipe {
            first: 11798,
            second: hilt,
            output: out,
            skill: 13,
            level: 0,
            base_xp: 0,
            members: true,
        });
    }
    Ok(recipes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        assembly::{Job, Outcome},
        inventory::{Inventory, Item, StackType},
        stats::Stats,
    };
    use std::collections::BTreeMap;
    fn fixture() -> (Definitions, Symbols) {
        let mut defs = Definitions {
            inventories: BTreeMap::new(),
            locations: BTreeMap::new(),
            npcs: BTreeMap::new(),
            items: BTreeMap::new(),
            varbits: BTreeMap::new(),
            varps: BTreeMap::new(),
            stats: BTreeMap::new(),
        };
        let mut objects = Symbols::new();
        for (id, symbol, name) in [
            (11798, "godwars_godsword_blade1+2+3", "Godsword blade"),
            (11810, "godwars_godsword_hilt_armadyl", "Armadyl hilt"),
            (11802, "ags", "Armadyl godsword"),
            (11812, "godwars_godsword_hilt_bandos", "Bandos hilt"),
            (11804, "bgs", "Bandos godsword"),
            (11814, "godwars_godsword_hilt_saradomin", "Saradomin hilt"),
            (11806, "sgs", "Saradomin godsword"),
            (11816, "godwars_godsword_hilt_zamorak", "Zamorak hilt"),
            (11808, "zgs", "Zamorak godsword"),
            (26370, "godwars_godsword_hilt_ancient", "Ancient hilt"),
            (26233, "ancient_godsword", "Ancient godsword"),
        ] {
            objects.insert(symbol.into(), id);
            let mut raw = vec![2];
            raw.extend_from_slice(name.as_bytes());
            raw.extend_from_slice(b"\0\x10\0");
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&raw).unwrap(),
            );
        }
        (defs, objects)
    }
    #[test]
    fn godsword_five_full_bag_attachments_have_no_skill_or_xp_gate_and_roll_back_failed_credit() {
        let (mut defs, objects) = fixture();
        let recipes = godsword_attachments(&objects, &defs).unwrap();
        assert_eq!(recipes.len(), 5);
        let mut cat = defs.inventory_catalog();
        for recipe in recipes {
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
            for (slot, id) in [(21, 11798), (27, recipe.second), (2, recipe.second)] {
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
            let mut stats = Stats::default();
            stats.set_current(13, 0);
            stats.set_fine_xp(13, 12345).unwrap();
            let before = (inv.clone(), stats.clone());
            let job = Job {
                recipe,
                first_slot: 21,
                second_slot: 27,
            };
            assert_eq!(
                job.complete_awarding(&mut inv, &cat, &mut stats, 0, false)
                    .0,
                Outcome::Members
            );
            assert_eq!((inv.clone(), stats.clone()), before);
            cat.dummy.insert(recipe.output);
            assert_eq!(
                job.complete_awarding(&mut inv, &cat, &mut stats, 0, true).0,
                Outcome::Capacity
            );
            assert_eq!((inv.clone(), stats.clone()), before);
            cat.dummy.clear();
            let (result, advancement) = job.complete_awarding(&mut inv, &cat, &mut stats, 0, true);
            assert_eq!(result, Outcome::Made);
            assert!(advancement.is_none());
            assert_eq!(stats, before.1);
            assert_eq!(inv.get(2), before.0.get(2));
            assert_eq!(
                inv.slots()
                    .iter()
                    .flatten()
                    .filter(|i| i.id == recipe.output)
                    .count(),
                1
            );
            assert!(inv.get(27).is_none());
        }
        defs.items.get_mut(&26233).unwrap().cert.template = 799;
        assert!(godsword_attachments(&objects, &defs).is_err());
    }
    #[test]
    fn godsword_registered_held_attachment_is_immediate_and_never_replays() {
        use gielinor_world::{
            held_use::{Handler, Request, Scope, Type},
            simulation::{PlayerData, World, WorldEvent},
            CoordGrid,
        };
        let (defs, objects) = fixture();
        let recipes = godsword_attachments(&objects, &defs).unwrap();
        for case in 0..5 {
            let recipe = recipes[if case == 1 { 4 } else { 0 }];
            let mut w = World::default();
            w.collision.allocate(3200, 3200, 0);
            w.object_types = defs.inventory_catalog();
            for id in [recipe.first, recipe.second] {
                w.held_use_types.insert(id, Type { id, content: -1 });
            }
            w.held_use_handlers
                .register(
                    Scope::Types,
                    recipe.first,
                    recipe.second,
                    Handler::AssemblyImmediate(recipe),
                )
                .unwrap();
            let mut data = PlayerData::new(
                1,
                "Assembler".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                Stats::default(),
            );
            data.xp_rate_hundreds = 0;
            for (slot, id) in [(4, 11798), (17, recipe.second), (2, recipe.second)] {
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
            w.set_membership(a, case != 2).unwrap();
            w.drain_events();
            let stats = w.data(a).unwrap().stats.clone();
            if case == 3 {
                w.data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        17,
                        Some(Item {
                            id: recipe.second,
                            count: 1,
                            vars: 9,
                        }),
                    )
                    .unwrap();
            }
            if case == 4 {
                w.object_types.dummy.insert(recipe.output);
            }
            let inv = w.data(a).unwrap().inventory.clone();
            let request = if case == 1 {
                Request {
                    selected_slot: 17,
                    selected_id: recipe.second,
                    target_slot: 4,
                    target_id: recipe.first,
                }
            } else {
                Request {
                    selected_slot: 4,
                    selected_id: recipe.first,
                    target_slot: 17,
                    target_id: recipe.second,
                }
            };
            w.prepare_held_use(a, request).unwrap();
            w.finish_held_use(a, request).unwrap();
            let expected = case < 2;
            assert_eq!(
                w.data(a)
                    .unwrap()
                    .inventory
                    .slots()
                    .iter()
                    .flatten()
                    .any(|i| i.id == recipe.output),
                expected
            );
            if !expected {
                assert_eq!(w.data(a).unwrap().inventory, inv);
            }
            let events = w.drain_events();
            assert!(!events
                .iter()
                .any(|e| matches!(e, WorldEvent::SkillAdvanced { .. })));
            let traces = events
                .into_iter()
                .filter_map(|e| {
                    if let WorldEvent::AssemblyMade { trace, .. } = e {
                        Some(trace)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            assert_eq!(traces.len(), usize::from(expected));
            if expected {
                assert_eq!(traces[0].queued_at, traces[0].committed_at);
            }
            for _ in 0..3 {
                w.tick().unwrap();
            }
            assert_eq!(w.data(a).unwrap().stats, stats);
            assert_eq!(w.data(a).unwrap().inventory.get(2), inv.get(2));
            assert!(!w
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::AssemblyMade { .. })));
        }
    }
}
