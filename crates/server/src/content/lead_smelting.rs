//! Normal-furnace lead smelting; public facts adapted to the existing queued smithing engine.
use super::{Definitions, Symbols};
use gielinor_world::smelting::Recipe;

pub fn lead_smelting(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Recipe, String> {
    if stats.get("smithing") != Some(&13) {
        return Err("Missing Smithing stat".into());
    }
    let stat = defs
        .stats
        .get(&13)
        .ok_or("Missing Smithing stat definition")?;
    if stat.name != "Smithing" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Smithing stat".into());
    }
    for (symbol, id, name) in [
        ("lead_ore", 31716, "Lead ore"),
        ("lead_bar", 32889, "Lead bar"),
    ] {
        if objects.get(symbol).is_some_and(|v| *v != id) {
            return Err(format!("Conflicting direct lead alias {symbol}"));
        }
        let item = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing direct lead item")?;
        if item.name != name
            || !item.members
            || item.stackable
            || item.dummy == 1
            || item.cert.template != 0
            || item.placeholder.template != 0
            || item.transform.template != 0
        {
            return Err(format!("Invalid direct lead item {id}"));
        }
    }
    Ok(Recipe {
        bar: 32889,
        label: "Lead".into(),
        level: 25,
        xp: 15.5,
        ingredients: vec![(31716, 2)],
        impurity: false,
        members: true,
        // Reentrant weak queues decrement new jobs in the same logical tick.
        repeat_delay: Some(6),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        inventory::{Catalog, Inventory, Item, StackType},
        simulation::{PlayerData, World, WorldEvent},
        smithing_make::{Config, Outcome},
        stats::Stats,
        CoordGrid,
    };
    use std::sync::Arc;
    struct NoDraw;
    impl gielinor_world::npc::RandomSource for NoDraw {
        fn below(&mut self, _: u32) -> u32 {
            panic!("Lead must not roll iron impurity")
        }
    }
    fn fixture() -> (Definitions, Symbols) {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        d.stats.insert(
            13,
            gielinor_cache::definitions::StatType::decode(b"\x01Smithing\0\0").unwrap(),
        );
        for (id, name) in [(31716, "Lead ore"), (32889, "Lead bar")] {
            let mut bytes = vec![2];
            bytes.extend_from_slice(name.as_bytes());
            bytes.extend_from_slice(b"\0\x10\0");
            d.items.insert(
                id,
                gielinor_cache::definitions::ItemType::decode(&bytes).unwrap(),
            );
        }
        (d, [("smithing".into(), 13)].into())
    }
    fn request(d: &Definitions, n: &Symbols) -> Arc<gielinor_world::smithing_make::Request> {
        let recipe = lead_smelting(&Symbols::new(), n, d).unwrap();
        let mut stats = Stats::default();
        stats.set_current(13, 25);
        let mut inv = Inventory::new(28, StackType::Normal, false);
        for slot in 0..4 {
            inv.restore_slot(
                slot,
                Some(Item {
                    id: 31716,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let gielinor_world::smelting::Selection::Start { request, .. } = recipe
            .select(
                &inv,
                &d.inventory_catalog(),
                &crate::smelting::item_types(&d.items),
                &stats,
                13,
                899,
                2,
            )
            .unwrap()
        else {
            panic!("valid lead recipe")
        };
        assert!(request.members && !request.impurity);
        assert_eq!(request.consumed, vec![(31716, 2)]);
        request
    }
    #[test]
    fn lead_smelting_direct_definitions_aliases_and_furnace_ore_binding() {
        let (mut d, n) = fixture();
        let recipe = lead_smelting(&Symbols::new(), &n, &d).unwrap();
        assert_eq!((recipe.bar, recipe.level, recipe.xp), (32889, 25, 15.5));
        let aliases = [("lead_ore".into(), 31716), ("lead_bar".into(), 32889)].into();
        assert!(lead_smelting(&aliases, &n, &d).is_ok());
        let bad = [("lead_bar".into(), 31719)].into();
        assert!(lead_smelting(&bad, &n, &d).is_err());
        d.items.get_mut(&32889).unwrap().transform.template = 1;
        assert!(lead_smelting(&Symbols::new(), &n, &d).is_err());
        let ores = [31716].into();
        let ordinary = gielinor_world::smelting::Furnace {
            operations: 6,
            item_use: true,
            ore_ids: None,
            tutorial: None,
        };
        assert!(ordinary.accepts_ore(31716, &ores));
        let tutorial = gielinor_world::smelting::Furnace {
            ore_ids: Some([436, 438]),
            ..ordinary
        };
        assert!(!tutorial.accepts_ore(31716, &ores));
    }
    #[test]
    fn lead_smelting_queued_member_and_live_requirements_preserve_completed_bars() {
        let (defs, names) = fixture();
        // 0 normal, 1 F2P admission, 2 late member loss, 3 after first,
        // 4 late level drain, 5 late input loss, 6 rejected output credit.
        for refusal in 0..7 {
            let mut w = World::default();
            w.collision.allocate(3200, 3200, 0);
            w.object_types = defs.inventory_catalog();
            if refusal == 6 {
                w.object_types.dummy.insert(32889);
            }
            w.smithing = Some(Config {
                queue: 900,
                skill: 13,
                max_level: 99,
                xp_rate: 1.25,
                tutorial: None,
            });
            let mut stats = Stats::default();
            stats.set_current(13, 25);
            let mut data = PlayerData::new(
                1,
                "Lead smith".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                stats,
            );
            // Full bag remains productive: two nonstackable inputs free enough slots.
            for slot in 0..28 {
                data.inventory
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: if slot < 4 { 31716 } else { 1511 },
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
            w.queue_login(1, data).unwrap();
            w.tick().unwrap();
            let a = w.actors().next().unwrap();
            w.set_membership(a, refusal != 1).unwrap();
            w.drain_events();
            let started = w.clock.0;
            w.start_smithing(a, request(&defs, &names), 2).unwrap();
            if refusal == 2 {
                w.set_membership(a, false).unwrap();
            }
            if refusal == 4 {
                w.data_mut(a).unwrap().stats.set_current(13, 24);
            }
            if refusal == 5 {
                for slot in 0..3 {
                    w.data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(slot, None)
                        .unwrap();
                }
            }
            let before = w.data(a).unwrap().clone();
            let mut products = vec![];
            for _ in 0..12 {
                w.tick_with_random(&mut NoDraw).unwrap();
                for e in w.drain_events() {
                    if matches!(
                        e,
                        WorldEvent::SmithingProductMade {
                            product: 32889,
                            count: 1,
                            ..
                        }
                    ) {
                        products.push(w.clock.0 - 1);
                        if refusal == 3 {
                            w.set_membership(a, false).unwrap();
                        }
                    }
                }
            }
            let count = if refusal == 0 {
                2
            } else if refusal == 3 {
                1
            } else {
                0
            };
            let d = w.data(a).unwrap();
            assert_eq!(products.len(), count);
            assert_eq!(
                d.stats.fine_xp(13) - before.stats.fine_xp(13),
                193 * count as i32
            );
            if count > 0 {
                assert_eq!(products[0] - started, 4);
            }
            if count == 2 {
                assert_eq!(products[1] - products[0], 5);
            }
            if count == 0 {
                assert_eq!(d.inventory, before.inventory);
                assert_eq!(d.stats, before.stats);
            }
            assert_eq!(
                d.inventory
                    .slots()
                    .iter()
                    .flatten()
                    .filter(|i| i.id == 1511)
                    .count(),
                24
            );
        }
    }
    #[test]
    fn lead_smelting_legacy_completion_has_no_member_authority_and_invalid_xp_is_atomic() {
        let (d, n) = fixture();
        let r = request(&d, &n);
        let job = r.start(1).unwrap();
        let mut inv = Inventory::new(2, StackType::Normal, false);
        for slot in 0..2 {
            inv.restore_slot(
                slot,
                Some(Item {
                    id: 31716,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let before = inv.clone();
        let mut stats = Stats::default();
        stats.set_current(13, 25);
        assert!(matches!(
            job.complete(
                &mut inv,
                &Catalog::default(),
                &mut stats,
                13,
                1.0,
                99,
                &mut NoDraw
            )
            .unwrap(),
            Outcome::RequirementsChanged
        ));
        assert_eq!(inv, before);
        assert!(job
            .complete_for_member(
                &mut inv,
                &Catalog::default(),
                &mut stats,
                13,
                f64::INFINITY,
                99,
                true,
                &mut NoDraw
            )
            .is_err());
        assert_eq!(inv, before);
        assert_eq!(stats.fine_xp(13), 0);
        assert!(matches!(
            job.complete_for_member(
                &mut inv,
                &Catalog::default(),
                &mut stats,
                13,
                1.0,
                99,
                true,
                &mut NoDraw
            )
            .unwrap(),
            Outcome::Made { .. }
        ));
        assert_eq!(stats.fine_xp(13), 155);
    }
}
