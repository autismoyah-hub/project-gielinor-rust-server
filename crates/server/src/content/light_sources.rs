//! Inventory light toggles; fuel acquisition, cave safety and worn effects are separate.
use super::{Definitions, Symbols};
use gielinor_world::{held_item_conversion::Rule, tool_crafting::Recipe};
pub struct LightSource {
    pub light: Recipe,
    pub extinguish: Rule,
}
// Exact target item bindings and Extinguish index; revision239 differs from donor op2.
const ROWS: [(&str, i32, &str, &str, i32, &str, u8, usize, &str); 9] = [
    (
        "unlit_candle",
        36,
        "Candle",
        "lit_candle",
        33,
        "Lit candle",
        1,
        0,
        "candle",
    ),
    (
        "unlit_black_candle",
        38,
        "Black candle",
        "lit_black_candle",
        32,
        "Lit black candle",
        1,
        0,
        "black candle",
    ),
    (
        "torch_unlit",
        596,
        "Unlit torch",
        "torch_lit",
        594,
        "Lit torch",
        1,
        3,
        "torch",
    ),
    (
        "candle_lantern_unlit",
        4529,
        "Candle lantern",
        "candle_lantern_lit",
        4531,
        "Candle lantern",
        4,
        1,
        "candle lantern",
    ),
    (
        "candle_lantern_black_unlit",
        4532,
        "Candle lantern",
        "candle_lantern_black_lit",
        4534,
        "Candle lantern",
        4,
        1,
        "black candle lantern",
    ),
    (
        "oil_lamp_unlit",
        4522,
        "Oil lamp",
        "oil_lamp_lit",
        4524,
        "Oil lamp",
        12,
        1,
        "oil lamp",
    ),
    (
        "oil_lantern_unlit",
        4537,
        "Oil lantern",
        "oil_lantern_lit",
        4539,
        "Oil lantern",
        26,
        1,
        "oil lantern",
    ),
    (
        "bullseye_lantern_unlit",
        4548,
        "Bullseye lantern",
        "bullseye_lantern_lit",
        4550,
        "Bullseye lantern",
        49,
        1,
        "bullseye lantern",
    ),
    (
        "cave_goblin_mining_helmet_unlit",
        5014,
        "Mining helmet",
        "cave_goblin_mining_helmet_lit",
        5013,
        "Mining helmet",
        65,
        2,
        "mining helmet",
    ),
];
pub fn light_sources(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<LightSource>, String> {
    if stats.get("firemaking") != Some(&11) {
        return Err("Missing/aliased Firemaking stat".into());
    }
    let stat = defs.stats.get(&11).ok_or("Missing Firemaking stat")?;
    if stat.name != "Firemaking" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Firemaking stat".into());
    }
    let check = |symbol: &str, id: i32, name: &str, member: bool| -> Result<(), String> {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased light item {symbol}"));
        }
        let d = defs.items.get(&(id as u32)).ok_or("Missing light item")?;
        if d.name != name
            || d.members != member
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid light item {id}"));
        }
        Ok(())
    };
    check("tinderbox", 590, "Tinderbox", false)?;
    let mut profiles = Vec::new();
    for (us, unlit, un, ls, lit, ln, level, op, name) in ROWS {
        check(us, unlit, un, true)?;
        check(ls, lit, ln, true)?;
        if defs.items[&(lit as u32)].inventory_ops[op].as_deref() != Some("Extinguish") {
            return Err(format!("Invalid target Extinguish operation {lit}"));
        }
        profiles.push(LightSource {
            light: Recipe {
                skill: 11,
                exchange: gielinor_world::grinding::Recipe {
                    tool: 590,
                    input: unlit,
                    output: lit,
                    members: true,
                    animation: None,
                },
                level,
                xp_tenths: 0,
                output_count: 1,
                ticks: 0,
                name,
                action: "light",
                failure: None,
            },
            extinguish: Rule {
                output: unlit,
                members: true,
                button_operation: [2, 3, 4, 6, 7][op],
                message: Some("You extinguish the light source."),
            },
        });
    }
    Ok(profiles)
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        inventory::{Inventory, Item, StackType},
        stats::Stats,
        tool_crafting::Job,
    };
    struct NoDraw;
    impl gielinor_world::npc::RandomSource for NoDraw {
        fn below(&mut self, _: u32) -> u32 {
            panic!("Light toggles must not draw RNG")
        }
    }
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
        let mut add = |symbol: &str, id: i32, name: &str, member: bool, op: Option<usize>| {
            objects.insert(symbol.into(), id);
            let mut b = vec![2];
            b.extend_from_slice(name.as_bytes());
            b.push(0);
            if member {
                b.push(16);
            }
            if let Some(op) = op {
                b.push(35 + op as u8);
                b.extend_from_slice(b"Extinguish\0");
            }
            b.push(0);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&b).unwrap(),
            );
        };
        add("tinderbox", 590, "Tinderbox", false, None);
        for (us, u, un, ls, l, ln, _, op, _) in ROWS {
            add(us, u, un, true, None);
            add(ls, l, ln, true, Some(op));
        }
        (defs, objects, [("firemaking".into(), 11)].into())
    }
    #[test]
    fn light_sources_nine_full_bag_roundtrips_use_firemaking_and_preserve_tool_and_stats() {
        let (mut defs, obj, names) = fixture();
        let profiles = light_sources(&obj, &names, &defs).unwrap();
        assert_eq!(profiles.len(), 9);
        for (profile, operation) in profiles.into_iter().zip([2, 2, 6, 3, 3, 3, 3, 3, 4]) {
            let r = profile.light;
            assert_eq!(profile.extinguish.button_operation, operation);
            let cat = defs.inventory_catalog();
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
                0,
                Some(Item {
                    id: 590,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
            inv.restore_slot(
                19,
                Some(Item {
                    id: r.exchange.input,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
            let mut stats = Stats::default();
            stats.set_current(12, 0);
            stats.set_current(11, r.level as i8);
            stats.set_fine_xp(11, 12345).unwrap();
            let before = (inv.clone(), stats.clone());
            let job = Job {
                recipe: r,
                tool_slot: 0,
                input_slot: 19,
            };
            assert!(job.prepare(&inv, &cat, &stats, 0, false).is_err());
            stats.set_current(11, r.level as i8 - 1);
            assert!(job.prepare(&inv, &cat, &stats, 0, true).is_err());
            stats = before.1.clone();
            let mut bad = defs.inventory_catalog();
            bad.dummy.insert(r.exchange.output);
            assert!(job
                .resolve(&inv, &bad, &stats, 0, true, &mut NoDraw)
                .is_err());
            assert_eq!((inv.clone(), stats.clone()), before);
            let (prepared, success) = job
                .resolve(&inv, &cat, &stats, 0, true, &mut NoDraw)
                .unwrap();
            assert!(success);
            assert!(prepared.advancement.is_none());
            assert_eq!(prepared.stats, stats);
            assert_eq!(prepared.inventory.get(19).unwrap().id, r.exchange.output);
            assert_eq!(prepared.inventory.get(0), inv.get(0));
            let round = profile
                .extinguish
                .prepare(r.exchange.output, 19, &prepared.inventory, &cat, true)
                .unwrap();
            assert_eq!(round, inv);
        }
        defs.items.get_mut(&5013).unwrap().inventory_ops[2] = None;
        assert!(light_sources(&obj, &names, &defs).is_err());
    }
    #[test]
    fn light_sources_actual_immediate_both_orders_and_registered_held_extinguish_roundtrip() {
        use gielinor_world::{
            held_use::{Handler, Request, Scope, Type},
            simulation::{Action, PlayerData, World, WorldEvent},
            CoordGrid,
        };
        let (defs, obj, names) = fixture();
        for profile in light_sources(&obj, &names, &defs).unwrap() {
            for reverse in [false, true] {
                let r = profile.light;
                let mut w = World::default();
                w.collision.allocate(3200, 3200, 0);
                w.object_types = defs.inventory_catalog();
                for id in [590, r.exchange.input] {
                    w.held_use_types.insert(id, Type { id, content: -1 });
                }
                w.held_use_handlers
                    .register(
                        Scope::Types,
                        590,
                        r.exchange.input,
                        Handler::ToolCraftingImmediate(r),
                    )
                    .unwrap();
                w.empty_containers
                    .insert(r.exchange.output, profile.extinguish);
                let mut stats = Stats::default();
                stats.set_base(11, r.level as i8);
                stats.set_current(11, r.level as i8);
                stats.set_current(12, 0);
                let mut data = PlayerData::new(
                    1,
                    "Light keeper".into(),
                    CoordGrid::new(3200, 3200, 0).unwrap(),
                    stats,
                );
                data.xp_rate_hundreds = 0;
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
                for (slot, id) in [(0, 590), (19, r.exchange.input), (8, r.exchange.input)] {
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
                let before = w.data(a).unwrap().clone();
                let req = if reverse {
                    Request {
                        selected_slot: 19,
                        selected_id: r.exchange.input,
                        target_slot: 0,
                        target_id: 590,
                    }
                } else {
                    Request {
                        selected_slot: 0,
                        selected_id: 590,
                        target_slot: 19,
                        target_id: r.exchange.input,
                    }
                };
                w.prepare_held_use(a, req).unwrap();
                w.finish_held_use(a, req).unwrap();
                assert_eq!(
                    w.data(a).unwrap().inventory.get(19).unwrap().id,
                    r.exchange.output
                );
                assert_eq!(w.data(a).unwrap().stats, before.stats);
                assert_eq!(
                    w.held_conversion(r.exchange.output)
                        .unwrap()
                        .button_operation,
                    profile.extinguish.button_operation
                );
                w.data_mut(a).unwrap().stats.set_current(11, 0);
                let stats = w.data(a).unwrap().stats.clone();
                w.submit(
                    a,
                    Action::ConvertHeldItem {
                        slot: 19,
                        item: r.exchange.output,
                    },
                )
                .unwrap();
                w.tick().unwrap();
                assert_eq!(w.data(a).unwrap().inventory, before.inventory);
                assert_eq!(w.data(a).unwrap().stats, stats);
                let events = w.drain_events();
                assert!(!events
                    .iter()
                    .any(|e| matches!(e, WorldEvent::SkillAdvanced { .. })));
                assert_eq!(events.iter().filter(|e|matches!(e,WorldEvent::GameMessage{text,..}if text=="You extinguish the light source.")).count(),1);
            }
        }
    }
}
