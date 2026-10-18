//! Uncharged item assembly only; charging, combat, swapping and paid service are separate.
use super::{Definitions, Symbols};
use gielinor_world::assembly::{PreciseRecipe, Recipe};
pub fn accursed_upgrade(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<PreciseRecipe, String> {
    if stats.get("crafting") != Some(&12) {
        return Err("Missing/aliased Crafting stat".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    for (symbol, id, name) in [
        ("wbr_vetion_skull", 27673, "Skull of vet'ion"),
        (
            "wild_cave_sceptre_uncharged",
            22552,
            "Thammaron's sceptre (u)",
        ),
        (
            "wild_cave_accursed_uncharged",
            27662,
            "Accursed sceptre (u)",
        ),
    ] {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased accursed upgrade item {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing accursed upgrade item")?;
        // The pinned target retains the pre-August2026 lower-case skull name.
        if d.name != name
            || !d.members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid accursed upgrade item {id}"));
        }
    }
    Ok(PreciseRecipe {
        recipe: Recipe {
            first: 27673,
            second: 22552,
            output: 27662,
            skill: 12,
            level: 85,
            base_xp: 0,
            members: true,
        },
        xp_tenths: 0,
        ticks: 1,
        animation: None,
        spot: None,
    })
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
            12,
            gielinor_cache::definitions::StatType::decode(b"\x01Crafting\0\0").unwrap(),
        );
        let mut objects = Symbols::new();
        for (symbol, id, name) in [
            ("wbr_vetion_skull", 27673, "Skull of vet'ion"),
            (
                "wild_cave_sceptre_uncharged",
                22552,
                "Thammaron's sceptre (u)",
            ),
            (
                "wild_cave_accursed_uncharged",
                27662,
                "Accursed sceptre (u)",
            ),
        ] {
            objects.insert(symbol.into(), id);
            let mut b = vec![2];
            b.extend_from_slice(name.as_bytes());
            b.extend_from_slice(&[0, 16, 0]);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&b).unwrap(),
            );
        }
        (defs, objects, [("crafting".into(), 12)].into())
    }
    #[test]
    fn accursed_upgrade_full_bag_boosted_requirement_without_magic_gate_or_xp() {
        let (mut defs, objects, names) = fixture();
        let r = accursed_upgrade(&objects, &names, &defs).unwrap();
        assert_eq!((r.ticks, r.xp_tenths), (1, 0));
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
        for (slot, id) in [(4, 27673), (19, 22552), (8, 22552)] {
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
        stats.set_base(12, 1);
        stats.set_current(12, 85);
        stats.set_current(6, 0);
        stats.set_fine_xp(12, 12345).unwrap();
        let job = Job {
            recipe: r.recipe,
            first_slot: 4,
            second_slot: 19,
        };
        let cat = defs.inventory_catalog();
        let before = (inv.clone(), stats.clone());
        assert_eq!(
            job.complete_awarding_tenths(&mut inv, &cat, &mut stats, 0, false, 0)
                .0,
            Outcome::Members
        );
        assert_eq!((inv.clone(), stats.clone()), before);
        stats.set_current(12, 84);
        assert_eq!(
            job.complete_awarding_tenths(&mut inv, &cat, &mut stats, 0, true, 0)
                .0,
            Outcome::Level
        );
        stats = before.1.clone();
        assert_eq!(inv, before.0);
        let mut bad = defs.inventory_catalog();
        bad.dummy.insert(27662);
        assert_eq!(
            job.complete_awarding_tenths(&mut inv, &bad, &mut stats, 0, true, 0)
                .0,
            Outcome::Capacity
        );
        assert_eq!((inv.clone(), stats.clone()), before);
        let (out, event) = job.complete_awarding_tenths(&mut inv, &cat, &mut stats, 0, true, 0);
        assert_eq!(out, Outcome::Made);
        assert!(event.is_none());
        assert_eq!(stats, before.1);
        assert_eq!(inv.get(8).unwrap().id, 22552);
        assert_eq!(inv.free_space(), 1);
        assert_eq!(
            inv.slots()
                .iter()
                .flatten()
                .filter(|i| i.id == 27662)
                .count(),
            1
        );
        defs.items.get_mut(&22552).unwrap().cert.template = 799;
        assert!(accursed_upgrade(&objects, &names, &defs).is_err());
    }
    #[test]
    fn accursed_upgrade_actual_timed_pair_both_orders_live_refusals_and_no_repeat() {
        use gielinor_world::{
            held_use::{Handler, Request, Scope, Type},
            simulation::{PlayerData, World, WorldEvent},
            CoordGrid,
        };
        let (defs, objects, names) = fixture();
        let r = accursed_upgrade(&objects, &names, &defs).unwrap();
        for reverse in [false, true] {
            for refusal in 0..5 {
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
                stats.set_base(12, 85);
                stats.set_current(12, 85);
                stats.set_current(6, 0);
                let mut data = PlayerData::new(
                    1,
                    "Accursed assembler".into(),
                    CoordGrid::new(3200, 3200, 0).unwrap(),
                    stats,
                );
                data.xp_rate_hundreds = 0;
                for (slot, id) in [(4, 27673), (19, 22552), (8, 22552)] {
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
                let before = w.data(a).unwrap().inventory.clone();
                let stats = w.data(a).unwrap().stats.clone();
                let request = if reverse {
                    Request {
                        selected_slot: 19,
                        selected_id: 22552,
                        target_slot: 4,
                        target_id: 27673,
                    }
                } else {
                    Request {
                        selected_slot: 4,
                        selected_id: 27673,
                        target_slot: 19,
                        target_id: 22552,
                    }
                };
                w.prepare_held_use(a, request).unwrap();
                w.finish_held_use(a, request).unwrap();
                assert_eq!(w.data(a).unwrap().inventory, before);
                let start = w.clock.0;
                match refusal {
                    1 => w.set_membership(a, false).unwrap(),
                    2 => {
                        w.data_mut(a).unwrap().stats.set_current(12, 84);
                    }
                    3 => {
                        w.object_types.dummy.insert(27662);
                    }
                    4 => {
                        w.data_mut(a)
                            .unwrap()
                            .inventory
                            .restore_slot(
                                19,
                                Some(Item {
                                    id: 22552,
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
                        .any(|i| i.id == 27662);
                    if made && !seen {
                        commits.push(w.clock.0 - 1);
                        seen = true;
                    }
                }
                assert_eq!(seen, refusal == 0);
                if refusal == 0 {
                    assert_eq!(commits, vec![start + 1]);
                    assert_eq!(w.data(a).unwrap().stats, stats);
                    assert_eq!(w.data(a).unwrap().inventory.get(8).unwrap().id, 22552);
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
