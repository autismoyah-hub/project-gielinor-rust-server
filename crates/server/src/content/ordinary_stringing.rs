//! Ordinary symbol and rabbit-foot stringing; blessing and equipment are separate.
use super::{Definitions, Symbols};
use gielinor_world::assembly::Recipe;
pub fn ordinary_stringing(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<Recipe>, String> {
    if stats.get("crafting") != Some(&12) {
        return Err("Missing/aliased ordinary stringing stat".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    let check = |symbol: &str, id: i32, name: &str, members: bool| -> Result<(), String> {
        if objects.get(symbol) != Some(&id) {
            return Err(format!(
                "Missing/aliased ordinary stringing object {symbol}"
            ));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing stringing item")?;
        if d.name != name
            || d.members != members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid ordinary stringing item {id}"));
        }
        Ok(())
    };
    check("ball_of_wool", 1759, "Ball of wool", false)?;
    let mut out = Vec::new();
    for (input, input_symbol, input_name, output, output_symbol, output_name, level, member) in [
        (
            1714,
            "nostringstar",
            "Unstrung symbol",
            1716,
            "stringstar",
            "Unblessed symbol",
            1,
            false,
        ),
        (
            1720,
            "nostringsnake",
            "Unstrung emblem",
            1722,
            "stringsnake",
            "Unpowered symbol",
            1,
            true,
        ),
        (
            10134,
            "hunting_rabbit_foot",
            "Rabbit foot",
            10132,
            "hunting_strung_rabbit_foot",
            "Strung rabbit foot",
            37,
            true,
        ),
    ] {
        check(input_symbol, input, input_name, member)?;
        check(output_symbol, output, output_name, member)?;
        out.push(Recipe {
            first: 1759,
            second: input,
            output,
            skill: 12,
            level,
            base_xp: 4,
            members: member,
        });
    }
    Ok(out)
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
    fn fixture() -> (Definitions, Symbols, Symbols) {
        let mut defs = Definitions {
            inventories: BTreeMap::new(),
            locations: BTreeMap::new(),
            npcs: BTreeMap::new(),
            items: BTreeMap::new(),
            varbits: BTreeMap::new(),
            varps: BTreeMap::new(),
            stats: BTreeMap::new(),
        };
        defs.stats.insert(
            12,
            gielinor_cache::definitions::StatType::decode(b"\x01Crafting\0\0").unwrap(),
        );
        let mut objects = Symbols::new();
        for (id, symbol, name, member) in [
            (1759, "ball_of_wool", "Ball of wool", false),
            (1714, "nostringstar", "Unstrung symbol", false),
            (1716, "stringstar", "Unblessed symbol", false),
            (1720, "nostringsnake", "Unstrung emblem", true),
            (1722, "stringsnake", "Unpowered symbol", true),
            (10134, "hunting_rabbit_foot", "Rabbit foot", true),
            (
                10132,
                "hunting_strung_rabbit_foot",
                "Strung rabbit foot",
                true,
            ),
        ] {
            objects.insert(symbol.into(), id);
            let mut raw = vec![2];
            raw.extend_from_slice(name.as_bytes());
            raw.push(0);
            if member {
                raw.push(16)
            }
            raw.push(0);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&raw).unwrap(),
            );
        }
        (defs, objects, [("crafting".into(), 12)].into())
    }
    #[test]
    fn ordinary_stringing_three_pairs_conserve_full_bag_and_award_only_crafting() {
        let (mut defs, objects, stat) = fixture();
        let recipes = ordinary_stringing(&objects, &stat, &defs).unwrap();
        assert_eq!(recipes.len(), 3);
        let catalog = defs.inventory_catalog();
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
            for (slot, id) in [(9, 1759), (25, recipe.second), (2, recipe.second)] {
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
            stats.set_base(12, recipe.level as i8);
            stats.set_current(12, recipe.level as i8);
            let before = (inv.clone(), stats.clone());
            let job = Job {
                recipe,
                first_slot: 9,
                second_slot: 25,
            };
            if recipe.members {
                assert_eq!(
                    job.complete_awarding(&mut inv, &catalog, &mut stats, 125, false)
                        .0,
                    Outcome::Members
                );
                assert_eq!((inv.clone(), stats.clone()), before);
            }
            assert_eq!(
                job.complete_awarding(&mut inv, &catalog, &mut stats, 125, true)
                    .0,
                Outcome::Made
            );
            let mut expected = before.1.clone();
            expected.set_fine_xp(12, 50).unwrap();
            assert_eq!(stats, expected);
            assert_eq!(stats.current(21), 1);
            assert_eq!(inv.get(2), before.0.get(2));
            assert_eq!(
                inv.slots()
                    .iter()
                    .flatten()
                    .filter(|v| v.id == recipe.output)
                    .count(),
                1
            );
            assert!(inv.slots().iter().flatten().all(|v| v.id != 1759));
        }
        defs.items.get_mut(&10132).unwrap().members = false;
        assert!(ordinary_stringing(&objects, &stat, &defs).is_err());
    }
    #[test]
    fn ordinary_stringing_actual_rabbit_route_checks_crafting_not_hunter_and_rechecks_live_level() {
        use gielinor_world::{
            held_use::{Handler, Request, Scope, Type},
            simulation::{PlayerData, World, WorldEvent},
            CoordGrid,
        };
        let (defs, objects, stat) = fixture();
        let recipe = ordinary_stringing(&objects, &stat, &defs).unwrap()[2];
        for case in 0..5 {
            let mut w = World::default();
            w.collision.allocate(3200, 3200, 0);
            w.object_types = defs.inventory_catalog();
            for id in [1759, 10134] {
                w.held_use_types.insert(id, Type { id, content: -1 });
            }
            w.held_use_handlers
                .register(
                    Scope::Types,
                    1759,
                    10134,
                    Handler::AssemblyTimed { recipe, ticks: 1 },
                )
                .unwrap();
            let mut stats = Stats::default();
            stats.set_base(12, 37);
            stats.set_current(12, if case == 1 { 36 } else { 37 });
            let mut data = PlayerData::new(
                1,
                "Stringer".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                stats,
            );
            data.xp_rate_hundreds = 125;
            for (slot, id) in [(3, 1759), (19, 10134), (1, 10134)] {
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
            let req = Request {
                selected_slot: 19,
                selected_id: 10134,
                target_slot: 3,
                target_id: 1759,
            };
            w.prepare_held_use(a, req).unwrap();
            w.finish_held_use(a, req).unwrap();
            if case == 3 {
                w.data_mut(a).unwrap().stats.set_current(12, 36);
            }
            if case == 4 {
                w.set_modal_open(a, true).unwrap();
                w.set_modal_open(a, false).unwrap();
            }
            w.tick().unwrap();
            w.tick().unwrap();
            assert_eq!(
                w.data(a).unwrap().stats.fine_xp(12),
                if case == 0 { 50 } else { 0 }
            );
            assert_eq!(w.data(a).unwrap().stats.current(21), 1);
            assert_eq!(w.data(a).unwrap().inventory.get(1).unwrap().id, 10134);
            let events = w.drain_events();
            if case == 1 || case == 3 {
                assert!(events.iter().any(|e| matches!(
                    e,
                    WorldEvent::LocationMessage {
                        message: "You need a higher Crafting level to make that.",
                        ..
                    }
                )));
            }
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(e, WorldEvent::AssemblyMade { .. }))
                    .count(),
                usize::from(case == 0)
            );
        }
    }
}
