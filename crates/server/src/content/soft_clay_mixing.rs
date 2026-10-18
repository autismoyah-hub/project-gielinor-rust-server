//! Ordinary clay softening; magic, mining and facility use are separate.
use super::{Definitions, Symbols};
use gielinor_world::assembly::BatchRecipe;
pub fn soft_clay_mixing(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<BatchRecipe>, String> {
    if stats.get("crafting") != Some(&12) {
        return Err("Missing/aliased Crafting stat".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    for (symbol, id, name) in [
        ("clay", 434, "Clay"),
        ("softclay", 1761, "Soft clay"),
        ("bucket_water", 1929, "Bucket of water"),
        ("bucket_empty", 1925, "Bucket"),
        ("jug_water", 1937, "Jug of water"),
        ("jug_empty", 1935, "Jug"),
        ("bowl_water", 1921, "Bowl of water"),
        ("bowl_empty", 1923, "Bowl"),
        ("vial_water", 227, "Vial of water"),
        ("vial_empty", 229, "Vial"),
    ] {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased clay mixing item {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing clay mixing definition")?;
        if d.name != name
            || d.members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid clay mixing item {id}"));
        }
    }
    Ok([(1929, 1925), (1937, 1935), (1921, 1923), (227, 229)]
        .into_iter()
        .map(|(water, empty)| BatchRecipe {
            inputs: vec![434, water],
            outputs: vec![1761, empty],
            skill: 12,
            level: 1,
            base_xp: 0,
            members: false,
            ticks: 2,
        })
        .collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        held_use::{Handler, Request, Scope, Type},
        inventory::Item,
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
        let mut obj = Symbols::new();
        for (id, symbol, name) in [
            (434, "clay", "Clay"),
            (1761, "softclay", "Soft clay"),
            (1929, "bucket_water", "Bucket of water"),
            (1925, "bucket_empty", "Bucket"),
            (1937, "jug_water", "Jug of water"),
            (1935, "jug_empty", "Jug"),
            (1921, "bowl_water", "Bowl of water"),
            (1923, "bowl_empty", "Bowl"),
            (227, "vial_water", "Vial of water"),
            (229, "vial_empty", "Vial"),
        ] {
            obj.insert(symbol.into(), id);
            let mut b = vec![2];
            b.extend_from_slice(name.as_bytes());
            b.extend_from_slice(&[0, 0]);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&b).unwrap(),
            );
        }
        (defs, obj, [("crafting".into(), 12)].into())
    }
    #[test]
    fn soft_clay_mixing_four_actual_held_pairs_return_container_with_zero_xp_in_full_bag() {
        let (defs, obj, stat) = fixture();
        let recipes = soft_clay_mixing(&obj, &stat, &defs).unwrap();
        assert_eq!(recipes.len(), 4);
        for recipe in recipes {
            let water = recipe.inputs[1];
            let empty = recipe.outputs[1];
            let mut w = World::default();
            w.object_types = defs.inventory_catalog();
            w.collision.allocate(3200, 3200, 0);
            let mut stats = Stats::default();
            stats.set_base(3, 10);
            stats.set_current(3, 10);
            let mut data = PlayerData::new(
                1,
                "Clay mixer".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                stats.clone(),
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
            for (slot, id) in [(5, water), (19, 434), (21, 434)] {
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
            for id in [434, water] {
                w.held_use_types.insert(id, Type { id, content: -1 });
            }
            w.held_use_handlers
                .register(
                    Scope::Types,
                    434,
                    water,
                    Handler::AssemblyBatch(std::sync::Arc::new(recipe)),
                )
                .unwrap();
            let held = Request {
                selected_slot: 5,
                selected_id: water,
                target_slot: 19,
                target_id: 434,
            };
            w.prepare_held_use(a, held).unwrap();
            w.finish_held_use(a, held).unwrap();
            for _ in 0..6 {
                w.tick().unwrap();
            }
            let d = w.data(a).unwrap();
            assert_eq!(d.stats, stats);
            assert_eq!(d.inventory.get(21).unwrap().id, 434);
            assert_eq!(d.inventory.free_space(), 0);
            for id in [1761, empty] {
                assert_eq!(
                    d.inventory
                        .slots()
                        .iter()
                        .flatten()
                        .filter(|i| i.id == id)
                        .count(),
                    1
                );
            }
            assert!(!w
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::SkillAdvanced { .. })));
        }
    }
    #[test]
    fn soft_clay_mixing_failed_second_credit_stale_material_and_live_level_never_consume_inputs() {
        let (mut defs, obj, stat) = fixture();
        for case in 0..3 {
            let recipe =
                std::sync::Arc::new(soft_clay_mixing(&obj, &stat, &defs).unwrap().remove(0));
            let mut inv = gielinor_world::inventory::Inventory::new(
                2,
                gielinor_world::inventory::StackType::Normal,
                false,
            );
            for (slot, id) in [(0, 434), (1, 1929)] {
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
            let held = Request {
                selected_slot: 0,
                selected_id: 434,
                target_slot: 1,
                target_id: 1929,
            };
            let job = recipe.bind(held, &inv).unwrap();
            let mut cat = defs.inventory_catalog();
            let mut stats = Stats::default();
            match case {
                0 => {
                    cat.dummy.insert(1925);
                }
                1 => {
                    inv.restore_slot(
                        1,
                        Some(Item {
                            id: 1929,
                            count: 1,
                            vars: 7,
                        }),
                    )
                    .unwrap();
                }
                _ => stats.set_current(12, 0),
            }
            let before = (inv.clone(), stats.clone());
            let (outcome, advanced) = job.complete_awarding(&mut inv, &cat, &mut stats, 0, false);
            assert_ne!(outcome, gielinor_world::assembly::Outcome::Made);
            assert!(advanced.is_none());
            assert_eq!((inv, stats), before);
        }
        defs.items.get_mut(&229).unwrap().cert.template = 799;
        assert!(soft_clay_mixing(&obj, &stat, &defs).is_err());
    }
}
