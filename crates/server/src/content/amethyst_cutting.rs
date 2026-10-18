//! Four independently pinned amethyst cuts with checked stack output counts.
use super::{Definitions, Symbols};
use gielinor_world::{grinding, tool_crafting::Recipe};
pub fn amethyst_cutting(
    objects: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<Recipe>, String> {
    if stats.get("crafting") != Some(&12) || sequences.get("human_amethystcutting") != Some(&6295) {
        return Err("Missing/aliased amethyst stat/sequence".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    let check =
        |symbol: &str, id: i32, name: &str, member: bool, stack: bool| -> Result<(), String> {
            if objects.get(symbol) != Some(&id) {
                return Err(format!("Missing/aliased amethyst item {symbol}"));
            }
            let d = defs
                .items
                .get(&(id as u32))
                .ok_or("Missing amethyst definition")?;
            if d.name != name
                || d.members != member
                || d.stackable != stack
                || d.dummy == 1
                || d.cert.template != 0
                || d.placeholder.template != 0
                || d.transform.template != 0
            {
                return Err(format!("Invalid amethyst item {id}"));
            }
            Ok(())
        };
    check("chisel", 1755, "Chisel", false, false)?;
    check("amethyst", 21347, "Amethyst", true, false)?;
    let mut out = Vec::new();
    for (symbol, id, name, level, output_count) in [
        (
            "xbows_bolt_tips_amethyst",
            21338,
            "Amethyst bolt tips",
            83,
            15,
        ),
        ("amethyst_arrowheads", 21350, "Amethyst arrowtips", 85, 15),
        (
            "amethyst_javelin_head",
            21352,
            "Amethyst javelin tips",
            87,
            5,
        ),
        ("amethyst_dart_tip", 25853, "Amethyst dart tip", 89, 8),
    ] {
        check(symbol, id, name, true, true)?;
        out.push(Recipe {
            exchange: grinding::Recipe {
                tool: 1755,
                input: 21347,
                output: id,
                members: true,
                animation: Some(6295),
            },
            level,
            xp_tenths: 600,
            output_count,
            ticks: 1,
            name,
            action: "cut",
            failure: None,
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
        simulation::{PlayerData, World},
        stats::Stats,
        tool_crafting::Job,
        CoordGrid,
    };
    struct NoDraw;
    impl RandomSource for NoDraw {
        fn below(&mut self, _: u32) -> u32 {
            panic!("Deterministic amethyst cutting must not draw RNG")
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
        for (id, symbol, name, member, stack) in [
            (1755, "chisel", "Chisel", false, false),
            (21347, "amethyst", "Amethyst", true, false),
            (
                21338,
                "xbows_bolt_tips_amethyst",
                "Amethyst bolt tips",
                true,
                true,
            ),
            (
                21350,
                "amethyst_arrowheads",
                "Amethyst arrowtips",
                true,
                true,
            ),
            (
                21352,
                "amethyst_javelin_head",
                "Amethyst javelin tips",
                true,
                true,
            ),
            (25853, "amethyst_dart_tip", "Amethyst dart tip", true, true),
        ] {
            obj.insert(symbol.into(), id);
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
        (
            defs,
            obj,
            [("human_amethystcutting".into(), 6295)].into(),
            [("crafting".into(), 12)].into(),
        )
    }
    #[test]
    fn amethyst_cutting_four_counts_award_per_raw_not_per_tip_and_reject_stack_overflow_atomically()
    {
        let (mut defs, obj, seq, stat) = fixture();
        let recipes = amethyst_cutting(&obj, &seq, &stat, &defs).unwrap();
        assert_eq!(recipes.len(), 4);
        let cat = defs.inventory_catalog();
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
            for (slot, id) in [(5, 1755), (19, 21347)] {
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
            let job = Job {
                recipe,
                tool_slot: 5,
                input_slot: 19,
            };
            let (p, success) = job
                .resolve(&inv, &cat, &stats, 125, true, &mut NoDraw)
                .unwrap();
            assert!(success);
            assert_eq!(
                p.inventory.get(19).unwrap().count,
                i32::from(recipe.output_count)
            );
            assert_eq!(p.inventory.get(19).unwrap().id, recipe.exchange.output);
            assert_eq!(p.inventory.get(5), inv.get(5));
            assert_eq!(p.stats.fine_xp(12), 750);
            inv.restore_slot(
                0,
                Some(Item {
                    id: recipe.exchange.output,
                    count: i32::MAX - i32::from(recipe.output_count) + 1,
                    vars: 0,
                }),
            )
            .unwrap();
            let before = (inv.clone(), stats.clone());
            assert!(job
                .resolve(&inv, &cat, &stats, 125, true, &mut NoDraw)
                .is_err());
            assert_eq!((inv, stats), before);
        }
        defs.items.get_mut(&25853).unwrap().stackable = false;
        assert!(amethyst_cutting(&obj, &seq, &stat, &defs).is_err());
    }
    #[test]
    fn amethyst_cutting_actual_four_choice_batch_stops_at_late_overflow_without_raw_or_xp_loss() {
        for overflow in [false, true] {
            let (defs, obj, seq, stat) = fixture();
            let recipes = amethyst_cutting(&obj, &seq, &stat, &defs).unwrap();
            let mut w = World::default();
            w.object_types = defs.inventory_catalog();
            w.collision.allocate(3200, 3200, 0);
            let mut stats = Stats::default();
            stats.set_base(3, 10);
            stats.set_current(3, 10);
            stats.set_base(12, 89);
            stats.set_current(12, 89);
            let mut data = PlayerData::new(
                1,
                "Amethyst cutter".into(),
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
            for (slot, id) in [(5, 1755), (19, 21347), (3, 21347), (11, 21347)] {
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
            data.inventory
                .restore_slot(
                    0,
                    Some(Item {
                        id: 25853,
                        count: if overflow { i32::MAX - 8 } else { 10 },
                        vars: 0,
                    }),
                )
                .unwrap();
            w.queue_login(1, data).unwrap();
            w.tick().unwrap();
            let a = w.actors().next().unwrap();
            w.set_membership(a, true).unwrap();
            for id in [1755, 21347] {
                w.held_use_types.insert(id, Type { id, content: -1 });
            }
            w.held_use_handlers
                .register(
                    Scope::Types,
                    1755,
                    21347,
                    Handler::ToolCraftingChoices {
                        recipes: std::sync::Arc::new(recipes),
                        cycle_ticks: 2,
                    },
                )
                .unwrap();
            let held = Request {
                selected_slot: 19,
                selected_id: 21347,
                target_slot: 5,
                target_id: 1755,
            };
            w.prepare_held_use(a, held).unwrap();
            w.finish_held_use(a, held).unwrap();
            let request = w.take_tool_crafting_request(a).unwrap().unwrap();
            assert_eq!(request.recipes.len(), 4);
            assert_eq!(request.available(&w.data(a).unwrap().inventory), 3);
            w.start_tool_crafting_choice(a, request, 3, 3).unwrap();
            for _ in 0..10 {
                w.tick_with_random(&mut NoDraw).unwrap();
            }
            let d = w.data(a).unwrap();
            assert_eq!(
                d.inventory.get(0).unwrap().count,
                if overflow { i32::MAX } else { 34 }
            );
            assert_eq!(d.stats.fine_xp(12), if overflow { 750 } else { 2250 });
            assert_eq!(d.inventory.get(5).unwrap().id, 1755);
            assert!(d.inventory.get(19).is_none());
            if overflow {
                assert_eq!(d.inventory.get(3).unwrap().id, 21347);
                assert_eq!(d.inventory.get(11).unwrap().id, 21347);
            } else {
                assert!(d.inventory.get(3).is_none());
                assert!(d.inventory.get(11).is_none());
            }
        }
    }
}
