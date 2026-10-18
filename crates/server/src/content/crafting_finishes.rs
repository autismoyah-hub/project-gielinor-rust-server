//! Manual finishes for already-owned empty light orbs and serpentine visages.
use super::{Definitions, Symbols};
use gielinor_world::{assembly, grinding, tool_crafting};
pub struct CraftingFinishes {
    pub light_orb: assembly::Recipe,
    pub serpentine_helm: tool_crafting::Recipe,
}
pub fn crafting_finishes(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<CraftingFinishes, String> {
    if stats.get("crafting") != Some(&12) {
        return Err("Missing/aliased Crafting stat".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    for (symbol, id, name, member) in [
        (
            "dorgesh_lightbulb_nofilament",
            10980,
            "Empty light orb",
            true,
        ),
        ("dorgesh_wire", 10981, "Cave goblin wire", true),
        ("dorgesh_light_bulb", 10973, "Light orb", true),
        ("chisel", 1755, "Chisel", false),
        ("serpentine_visage", 12927, "Serpentine visage", true),
        (
            "serpentine_helm",
            12929,
            "Serpentine helm (uncharged)",
            true,
        ),
    ] {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased Crafting finish item {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing Crafting finish definition")?;
        if d.name != name
            || d.members != member
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid Crafting finish item {id}"));
        }
    }
    Ok(CraftingFinishes {
        light_orb: assembly::Recipe {
            first: 10980,
            second: 10981,
            output: 10973,
            skill: 12,
            level: 87,
            base_xp: 104,
            members: true,
        },
        serpentine_helm: tool_crafting::Recipe {
            exchange: grinding::Recipe {
                tool: 1755,
                input: 12927,
                output: 12929,
                members: true,
                animation: None,
            },
            level: 52,
            xp_tenths: 1200,
            ticks: 1,
            name: "serpentine helm",
            action: "carve",
            output_count: 1,
            failure: None,
        },
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        held_use::{Handler, Request, Scope, Type},
        inventory::Item,
        simulation::{PlayerData, World},
        stats::Stats,
        CoordGrid,
    };
    fn definitions() -> (Definitions, Symbols, Symbols) {
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
            (
                10980,
                "dorgesh_lightbulb_nofilament",
                "Empty light orb",
                true,
            ),
            (10981, "dorgesh_wire", "Cave goblin wire", true),
            (10973, "dorgesh_light_bulb", "Light orb", true),
            (1755, "chisel", "Chisel", false),
            (12927, "serpentine_visage", "Serpentine visage", true),
            (
                12929,
                "serpentine_helm",
                "Serpentine helm (uncharged)",
                true,
            ),
        ] {
            obj.insert(symbol.into(), id);
            let mut b = vec![2];
            b.extend_from_slice(name.as_bytes());
            b.push(0);
            if member {
                b.push(16);
            }
            b.push(0);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&b).unwrap(),
            );
        }
        (defs, obj, [("crafting".into(), 12)].into())
    }
    #[test]
    fn crafting_finishes_actual_held_routes_keep_lamp_repair_and_charging_outside_manual_conversion(
    ) {
        let (defs, obj, stat) = definitions();
        for helm in [false, true] {
            let recipes = crafting_finishes(&obj, &stat, &defs).unwrap();
            let (mut w, level, first, second, output, fine, handler) = (
                World::default(),
                if helm { 52 } else { 87 },
                if helm { 12927 } else { 10980 },
                if helm { 1755 } else { 10981 },
                if helm { 12929 } else { 10973 },
                if helm { 1500 } else { 1300 },
                if helm {
                    Handler::ToolCrafting(recipes.serpentine_helm)
                } else {
                    Handler::AssemblyImmediate(recipes.light_orb)
                },
            );
            w.object_types = defs.inventory_catalog();
            w.collision.allocate(3200, 3200, 0);
            let mut stats = Stats::default();
            stats.set_base(3, 10);
            stats.set_current(3, 10);
            stats.set_base(12, level);
            stats.set_current(12, level);
            let mut data = PlayerData::new(
                1,
                "Finisher".into(),
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
            for (slot, id) in [(19, first), (5, second), (21, first), (23, second)] {
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
            for id in [first, second] {
                w.held_use_types.insert(id, Type { id, content: -1 });
            }
            w.held_use_handlers
                .register(Scope::Types, first, second, handler)
                .unwrap();
            let request = Request {
                selected_slot: 5,
                selected_id: second,
                target_slot: 19,
                target_id: first,
            };
            w.prepare_held_use(a, request).unwrap();
            w.finish_held_use(a, request).unwrap();
            assert_eq!(
                w.data(a).unwrap().stats.fine_xp(12),
                if helm { 0 } else { fine }
            );
            for _ in 0..6 {
                w.tick().unwrap();
            }
            let d = w.data(a).unwrap();
            assert_eq!(d.stats.fine_xp(12), fine);
            assert_eq!(d.stats.fine_xp(11), 0);
            assert_eq!(d.stats.current(1), 1);
            assert_eq!(
                d.inventory
                    .slots()
                    .iter()
                    .flatten()
                    .filter(|i| i.id == output && i.vars == 0)
                    .count(),
                1
            );
            assert_eq!(d.inventory.get(21).unwrap().id, first);
            assert_eq!(d.inventory.get(23).unwrap().id, second);
            if helm {
                assert_eq!(d.inventory.get(5).unwrap().id, 1755);
                assert_eq!(d.inventory.free_space(), 0);
            } else {
                assert_eq!(d.inventory.free_space(), 1);
            }
        }
    }
    #[test]
    fn crafting_finishes_require_exact_uncharged_outputs_and_live_membership_level_and_credit() {
        let (mut defs, obj, stat) = definitions();
        let recipes = crafting_finishes(&obj, &stat, &defs).unwrap();
        for case in 0..4 {
            let mut inv = gielinor_world::inventory::Inventory::new(
                2,
                gielinor_world::inventory::StackType::Normal,
                false,
            );
            for (slot, id) in [(0, 10980), (1, 10981)] {
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
            stats.set_base(12, 87);
            stats.set_current(12, 87);
            let mut cat = defs.inventory_catalog();
            let mut member = true;
            let mut rate = 125;
            match case {
                0 => member = false,
                1 => stats.set_current(12, 86),
                2 => {
                    cat.dummy.insert(10973);
                }
                _ => rate = 0,
            };
            let before = (inv.clone(), stats.clone());
            let (outcome, advanced) = assembly::Job {
                recipe: recipes.light_orb,
                first_slot: 0,
                second_slot: 1,
            }
            .complete_awarding(&mut inv, &cat, &mut stats, rate, member);
            assert_ne!(outcome, assembly::Outcome::Made);
            assert!(advanced.is_none());
            assert_eq!((inv, stats), before);
        }
        defs.items.get_mut(&12929).unwrap().name = "Serpentine helm".into();
        assert!(crafting_finishes(&obj, &stat, &defs).is_err());
    }
}
