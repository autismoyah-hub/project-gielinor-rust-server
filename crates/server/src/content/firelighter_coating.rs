//! Colour regular logs through existing immediate selected-pair assembly.
//! Ignition, campfires, phoenix recolouring and acquisition are separate content.
use super::{Definitions, Symbols};
use gielinor_world::assembly::Recipe;
pub fn firelighter_coating(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<Recipe>, String> {
    if stats.get("firemaking") != Some(&11) {
        return Err("Missing/aliased Firemaking stat".into());
    }
    let stat = defs.stats.get(&11).ok_or("Missing Firemaking stat")?;
    if stat.name != "Firemaking" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Firemaking stat".into());
    }
    let check =
        |symbol: &str, id: i32, name: &str, member: bool, stackable: bool| -> Result<(), String> {
            if objects.get(symbol) != Some(&id) {
                return Err(format!("Missing/aliased firelighter item {symbol}"));
            }
            let d = defs
                .items
                .get(&(id as u32))
                .ok_or("Missing firelighter item")?;
            if d.name != name
                || d.members != member
                || d.stackable != stackable
                || d.dummy == 1
                || d.cert.template != 0
                || d.placeholder.template != 0
                || d.transform.template != 0
            {
                return Err(format!("Invalid firelighter item {id}"));
            }
            Ok(())
        };
    check("logs", 1511, "Logs", false, false)?;
    let mut recipes = Vec::new();
    for (lighter_symbol, lighter, lighter_name, log_symbol, output, log_name) in [
        (
            "gnomish_firelighter_red",
            7329,
            "Red firelighter",
            "red_logs",
            7404,
            "Red logs",
        ),
        (
            "gnomish_firelighter_green",
            7330,
            "Green firelighter",
            "green_logs",
            7405,
            "Green logs",
        ),
        (
            "gnomish_firelighter_blue",
            7331,
            "Blue firelighter",
            "blue_logs",
            7406,
            "Blue logs",
        ),
        (
            "trail_gnomish_firelighter_purple",
            10326,
            "Purple firelighter",
            "trail_logs_purple",
            10329,
            "Purple logs",
        ),
        (
            "trail_gnomish_firelighter_white",
            10327,
            "White firelighter",
            "trail_logs_white",
            10328,
            "White logs",
        ),
    ] {
        check(lighter_symbol, lighter, lighter_name, true, true)?;
        check(log_symbol, output, log_name, true, false)?;
        // Zero requirement/reward: coating does not train or require Firemaking.
        recipes.push(Recipe {
            first: 1511,
            second: lighter,
            output,
            skill: 11,
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
            11,
            gielinor_cache::definitions::StatType::decode(b"\x01Firemaking\0\0").unwrap(),
        );
        let mut objects = Symbols::new();
        for (symbol, id, name, member, stack) in [
            ("logs", 1511, "Logs", false, false),
            (
                "gnomish_firelighter_red",
                7329,
                "Red firelighter",
                true,
                true,
            ),
            (
                "gnomish_firelighter_green",
                7330,
                "Green firelighter",
                true,
                true,
            ),
            (
                "gnomish_firelighter_blue",
                7331,
                "Blue firelighter",
                true,
                true,
            ),
            (
                "trail_gnomish_firelighter_purple",
                10326,
                "Purple firelighter",
                true,
                true,
            ),
            (
                "trail_gnomish_firelighter_white",
                10327,
                "White firelighter",
                true,
                true,
            ),
            ("red_logs", 7404, "Red logs", true, false),
            ("green_logs", 7405, "Green logs", true, false),
            ("blue_logs", 7406, "Blue logs", true, false),
            ("trail_logs_purple", 10329, "Purple logs", true, false),
            ("trail_logs_white", 10328, "White logs", true, false),
        ] {
            objects.insert(symbol.into(), id);
            let mut b = vec![2];
            b.extend_from_slice(name.as_bytes());
            b.push(0);
            if member {
                b.push(16);
            }
            if stack {
                b.push(11);
            }
            b.push(0);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&b).unwrap(),
            );
        }
        (defs, objects, [("firemaking".into(), 11)].into())
    }
    #[test]
    fn firelighter_coating_five_full_bag_atomic_pairs_keep_stack_and_zero_xp_at_drained_zero() {
        let (mut defs, objects, stats_names) = fixture();
        let recipes = firelighter_coating(&objects, &stats_names, &defs).unwrap();
        assert_eq!(recipes.len(), 5);
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
            inv.restore_slot(
                19,
                Some(Item {
                    id: recipe.second,
                    count: 3,
                    vars: 0,
                }),
            )
            .unwrap();
            let mut stats = Stats::default();
            stats.set_current(11, 0);
            stats.set_fine_xp(11, 12345).unwrap();
            let before = (inv.clone(), stats.clone());
            let job = Job {
                recipe,
                first_slot: 4,
                second_slot: 19,
            };
            let cat = defs.inventory_catalog();
            assert_eq!(
                job.complete_awarding(&mut inv, &cat, &mut stats, 0, false)
                    .0,
                Outcome::Members
            );
            assert_eq!((inv.clone(), stats.clone()), before);
            let mut bad = defs.inventory_catalog();
            bad.dummy.insert(recipe.output);
            assert_eq!(
                job.complete_awarding(&mut inv, &bad, &mut stats, 0, true).0,
                Outcome::Capacity
            );
            assert_eq!((inv.clone(), stats.clone()), before);
            let (result, event) = job.complete_awarding(&mut inv, &cat, &mut stats, 0, true);
            assert_eq!(result, Outcome::Made);
            assert!(event.is_none());
            assert_eq!(stats, before.1);
            assert_eq!(inv.get(19).unwrap().count, 2);
            assert_eq!(inv.get(4).unwrap().id, recipe.output);
            assert_eq!(inv.get(3).unwrap().id, 1511);
            let after = inv.clone();
            assert_eq!(
                job.complete_awarding(&mut inv, &cat, &mut stats, 0, true).0,
                Outcome::Stale
            );
            assert_eq!(inv, after);
        }
        defs.items.get_mut(&10328).unwrap().members = false;
        assert!(firelighter_coating(&objects, &stats_names, &defs).is_err());
    }
    #[test]
    fn firelighter_coating_actual_immediate_held_registry_accepts_both_orders_without_repeat() {
        use gielinor_world::{
            held_use::{Handler, Request, Scope, Type},
            simulation::{PlayerData, World, WorldEvent},
            CoordGrid,
        };
        let (defs, objects, stats_names) = fixture();
        for recipe in firelighter_coating(&objects, &stats_names, &defs).unwrap() {
            for reverse in [false, true] {
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
                    "Firelighter".into(),
                    CoordGrid::new(3200, 3200, 0).unwrap(),
                    Stats::default(),
                );
                data.xp_rate_hundreds = 0;
                data.stats.set_current(11, 0);
                for (slot, id, count) in [(4, 1511, 1), (8, 1511, 1), (19, recipe.second, 2)] {
                    data.inventory
                        .restore_slot(slot, Some(Item { id, count, vars: 0 }))
                        .unwrap();
                }
                w.queue_login(1, data).unwrap();
                w.tick().unwrap();
                let a = w.actors().next().unwrap();
                w.set_membership(a, true).unwrap();
                w.drain_events();
                let stats = w.data(a).unwrap().stats.clone();
                let request = if reverse {
                    Request {
                        selected_slot: 19,
                        selected_id: recipe.second,
                        target_slot: 4,
                        target_id: 1511,
                    }
                } else {
                    Request {
                        selected_slot: 4,
                        selected_id: 1511,
                        target_slot: 19,
                        target_id: recipe.second,
                    }
                };
                w.prepare_held_use(a, request).unwrap();
                w.finish_held_use(a, request).unwrap();
                for _ in 0..4 {
                    w.tick().unwrap();
                }
                let inv = &w.data(a).unwrap().inventory;
                assert_eq!(
                    inv.slots()
                        .iter()
                        .flatten()
                        .filter(|i| i.id == recipe.output)
                        .count(),
                    1
                );
                assert_eq!(inv.get(8).unwrap().id, 1511);
                assert_eq!(inv.get(19).unwrap().count, 1);
                assert_eq!(w.data(a).unwrap().stats, stats);
                assert!(!w
                    .drain_events()
                    .iter()
                    .any(|e| matches!(e, WorldEvent::SkillAdvanced { .. })));
            }
        }
    }
}
