//! Owned crab-material carving with Wiki success curves and zero-XP broken outcomes.
use super::{Definitions, Symbols};
use gielinor_world::{
    grinding,
    tool_crafting::{Failure, Recipe},
};
pub fn crab_carving(
    objects: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<Recipe>, String> {
    if stats.get("crafting") != Some(&12) || sequences.get("human_snailshellcutting") != Some(&1280)
    {
        return Err("Missing/aliased crab crafting stat/sequence".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    let check = |symbol: &str, id: i32, name: &str, member: bool| -> Result<(), String> {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased crab item {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing crab definition")?;
        if d.name != name
            || d.members != member
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid crab item {id}"));
        }
        Ok(())
    };
    check("chisel", 1755, "Chisel", false)?;
    let mut out = Vec::new();
    for (raw, raw_symbol, raw_name, output, out_symbol, name, broken, broken_symbol, broken_name) in [
        (
            7538,
            "hundred_pirate_crab_shell_head",
            "Fresh crab shell",
            7539,
            "hundred_pirate_crab_shell_helm",
            "Crab helmet",
            7541,
            "crab_shell_broken",
            "Broken crab shell",
        ),
        (
            7536,
            "hundred_pirate_crab_shell_claw",
            "Fresh crab claw",
            7537,
            "hundred_pirate_crab_shell_gauntlet",
            "Crab claw",
            7540,
            "crab_claw_broken",
            "Broken crab claw",
        ),
    ] {
        check(raw_symbol, raw, raw_name, true)?;
        check(out_symbol, output, name, true)?;
        check(broken_symbol, broken, broken_name, true)?;
        out.push(Recipe {
            exchange: grinding::Recipe {
                tool: 1755,
                input: raw,
                output,
                members: true,
                animation: Some(1280),
            },
            level: 15,
            xp_tenths: 325,
            ticks: 3,
            name,
            action: "carve",
            failure: Some(Failure {
                output: broken,
                xp_tenths: 0,
                low: 40,
                high: 240,
                message: "You accidentally break the crab material.",
            }),
        });
    }
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        held_use::{Handler, Request, Scope, Type},
        inventory::{Inventory, Item, StackType},
        npc::RandomSource,
        simulation::{PlayerData, World, WorldEvent},
        stats::Stats,
        tool_crafting::Job,
        CoordGrid,
    };
    struct Draw {
        values: Vec<u32>,
        calls: usize,
    }
    impl RandomSource for Draw {
        fn below(&mut self, n: u32) -> u32 {
            if n != 256 {
                return 0;
            }
            let value = self.values[self.calls];
            self.calls += 1;
            value
        }
    }
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
        for (id, symbol, name, member) in [
            (1755, "chisel", "Chisel", false),
            (
                7538,
                "hundred_pirate_crab_shell_head",
                "Fresh crab shell",
                true,
            ),
            (7539, "hundred_pirate_crab_shell_helm", "Crab helmet", true),
            (7541, "crab_shell_broken", "Broken crab shell", true),
            (
                7536,
                "hundred_pirate_crab_shell_claw",
                "Fresh crab claw",
                true,
            ),
            (
                7537,
                "hundred_pirate_crab_shell_gauntlet",
                "Crab claw",
                true,
            ),
            (7540, "crab_claw_broken", "Broken crab claw", true),
        ] {
            obj.insert(symbol.into(), id);
            let mut bytes = vec![2];
            bytes.extend_from_slice(name.as_bytes());
            bytes.push(0);
            if member {
                bytes.push(16);
            }
            bytes.push(0);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&bytes).unwrap(),
            );
        }
        (
            defs,
            obj,
            [("human_snailshellcutting".into(), 1280)].into(),
            [("crafting".into(), 12)].into(),
        )
    }
    #[test]
    fn crab_carving_boundaries_break_material_without_xp_or_fabricated_advancement() {
        let (mut defs, obj, seq, stat) = fixture();
        let recipes = crab_carving(&obj, &seq, &stat, &defs).unwrap();
        assert_eq!(recipes.len(), 2);
        let cat = defs.inventory_catalog();
        for recipe in recipes {
            let mut inv = Inventory::new(2, StackType::Normal, false);
            for (slot, id) in [(0, 1755), (1, recipe.exchange.input)] {
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
            stats.set_base(12, 15);
            stats.set_current(12, 15);
            let job = Job {
                recipe,
                tool_slot: 0,
                input_slot: 1,
            };
            for (level, roll, success) in [
                (15, 69, true),
                (15, 70, false),
                (99, 240, true),
                (99, 241, false),
            ] {
                stats.set_current(12, level);
                let mut rng = Draw {
                    values: vec![roll],
                    calls: 0,
                };
                let (p, actual) = job
                    .resolve(&inv, &cat, &stats, 125, true, &mut rng)
                    .unwrap();
                assert_eq!(actual, success);
                assert_eq!(rng.calls, 1);
                assert_eq!(p.inventory.get(0), inv.get(0));
                assert_eq!(
                    p.inventory.get(1).unwrap().id,
                    if success {
                        recipe.exchange.output
                    } else {
                        recipe.failure.unwrap().output
                    }
                );
                if success {
                    assert_eq!(p.stats.fine_xp(12), 406);
                    assert!(p.advancement.is_some());
                } else {
                    assert_eq!(p.stats, stats);
                    assert!(p.advancement.is_none());
                }
            }
            let mut invalid = defs.inventory_catalog();
            invalid.dummy.insert(recipe.failure.unwrap().output);
            let mut rng = Draw {
                values: vec![],
                calls: 0,
            };
            assert!(job
                .resolve(&inv, &invalid, &stats, 125, true, &mut rng)
                .is_err());
            assert_eq!(rng.calls, 0);
        }
        defs.items.get_mut(&7540).unwrap().cert.template = 799;
        assert!(crab_carving(&obj, &seq, &stat, &defs).is_err());
    }
    #[test]
    fn crab_carving_actual_choice_batch_commits_success_then_zero_xp_break() {
        let (defs, obj, seq, stat) = fixture();
        let recipe = crab_carving(&obj, &seq, &stat, &defs).unwrap()[0];
        let mut w = World::default();
        w.object_types = defs.inventory_catalog();
        w.collision.allocate(3200, 3200, 0);
        let mut stats = Stats::default();
        stats.set_base(3, 10);
        stats.set_current(3, 10);
        stats.set_base(12, 15);
        stats.set_current(12, 15);
        let mut data = PlayerData::new(
            1,
            "Crab carver".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            stats,
        );
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
        for (slot, id) in [(5, 1755), (19, 7538), (21, 7538)] {
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
        for id in [1755, 7538] {
            w.held_use_types.insert(id, Type { id, content: -1 });
        }
        w.held_use_handlers
            .register(
                Scope::Types,
                1755,
                7538,
                Handler::ToolCraftingChoices {
                    recipes: std::sync::Arc::new(vec![recipe]),
                    cycle_ticks: 3,
                },
            )
            .unwrap();
        let held = Request {
            selected_id: 7538,
            selected_slot: 19,
            target_id: 1755,
            target_slot: 5,
        };
        w.prepare_held_use(a, held).unwrap();
        w.finish_held_use(a, held).unwrap();
        let req = w.take_tool_crafting_request(a).unwrap().unwrap();
        w.start_tool_crafting_choice(a, req, 0, 2).unwrap();
        let mut rng = Draw {
            values: vec![0, 255],
            calls: 0,
        };
        for _ in 0..10 {
            w.tick_with_random(&mut rng).unwrap();
        }
        assert_eq!(rng.calls, 2);
        let d = w.data(a).unwrap();
        assert_eq!(d.inventory.get(19).unwrap().id, 7539);
        assert_eq!(d.inventory.get(21).unwrap().id, 7541);
        assert_eq!(d.inventory.get(5).unwrap().id, 1755);
        assert_eq!(d.stats.fine_xp(12), 406);
        assert_eq!(d.inventory.free_space(), 0);
        assert_eq!(
            w.drain_events()
                .iter()
                .filter(|e| matches!(e, WorldEvent::SkillAdvanced { skill: 12, .. }))
                .count(),
            1
        );
    }
}
