//! Manual amulet stringing; manufacture and enchantment are separate.
use super::{Definitions, Symbols};
use gielinor_world::assembly::Recipe;
pub fn amulet_stringing(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<Recipe>, String> {
    if stats.get("crafting") != Some(&12) {
        return Err("Missing/aliased stringing stat".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    let check = |symbol: &str, id: i32, name: &str, members: bool| -> Result<(), String> {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased stringing object {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing amulet stringing item")?;
        if d.name != name
            || d.members != members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid stringing definition {id}"));
        }
        Ok(())
    };
    check("ball_of_wool", 1759, "Ball of wool", false)?;
    let mut recipes = Vec::new();
    for (raw_symbol, raw, out_symbol, out, name, member) in [
        (
            "unstrung_gold_amulet",
            1673,
            "strung_gold_amulet",
            1692,
            "Gold amulet",
            false,
        ),
        (
            "unstrung_sapphire_amulet",
            1675,
            "strung_sapphire_amulet",
            1694,
            "Sapphire amulet",
            false,
        ),
        (
            "unstrung_emerald_amulet",
            1677,
            "strung_emerald_amulet",
            1696,
            "Emerald amulet",
            false,
        ),
        (
            "unstrung_ruby_amulet",
            1679,
            "strung_ruby_amulet",
            1698,
            "Ruby amulet",
            false,
        ),
        (
            "unstrung_diamond_amulet",
            1681,
            "strung_diamond_amulet",
            1700,
            "Diamond amulet",
            false,
        ),
        (
            "unstrung_dragonstone_amulet",
            1683,
            "strung_dragonstone_amulet",
            1702,
            "Dragonstone amulet",
            true,
        ),
        (
            "unstrung_onyx_amulet",
            6579,
            "strung_onyx_amulet",
            6581,
            "Onyx amulet",
            true,
        ),
        (
            "unstrung_zenyte_amulet",
            19501,
            "zenyte_amulet",
            19541,
            "Zenyte amulet",
            true,
        ),
        (
            "unstrung_opal_amulet",
            21099,
            "strung_opal_amulet",
            21108,
            "Opal amulet",
            true,
        ),
        (
            "unstrung_jade_amulet",
            21102,
            "strung_jade_amulet",
            21111,
            "Jade amulet",
            true,
        ),
        (
            "unstrung_topaz_amulet",
            21105,
            "strung_topaz_amulet",
            21114,
            "Topaz amulet",
            true,
        ),
    ] {
        check(raw_symbol, raw, &format!("{name} (u)"), member)?;
        check(out_symbol, out, name, member)?;
        recipes.push(Recipe {
            first: 1759,
            second: raw,
            output: out,
            skill: 12,
            level: 1,
            base_xp: 4,
            members: member,
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
            (1692, "strung_gold_amulet", "Gold amulet", false),
            (1673, "unstrung_gold_amulet", "Gold amulet (u)", false),
            (1694, "strung_sapphire_amulet", "Sapphire amulet", false),
            (
                1675,
                "unstrung_sapphire_amulet",
                "Sapphire amulet (u)",
                false,
            ),
            (1696, "strung_emerald_amulet", "Emerald amulet", false),
            (1677, "unstrung_emerald_amulet", "Emerald amulet (u)", false),
            (1698, "strung_ruby_amulet", "Ruby amulet", false),
            (1679, "unstrung_ruby_amulet", "Ruby amulet (u)", false),
            (1700, "strung_diamond_amulet", "Diamond amulet", false),
            (1681, "unstrung_diamond_amulet", "Diamond amulet (u)", false),
            (
                1702,
                "strung_dragonstone_amulet",
                "Dragonstone amulet",
                true,
            ),
            (
                1683,
                "unstrung_dragonstone_amulet",
                "Dragonstone amulet (u)",
                true,
            ),
            (6581, "strung_onyx_amulet", "Onyx amulet", true),
            (6579, "unstrung_onyx_amulet", "Onyx amulet (u)", true),
            (19541, "zenyte_amulet", "Zenyte amulet", true),
            (19501, "unstrung_zenyte_amulet", "Zenyte amulet (u)", true),
            (21108, "strung_opal_amulet", "Opal amulet", true),
            (21099, "unstrung_opal_amulet", "Opal amulet (u)", true),
            (21111, "strung_jade_amulet", "Jade amulet", true),
            (21102, "unstrung_jade_amulet", "Jade amulet (u)", true),
            (21114, "strung_topaz_amulet", "Topaz amulet", true),
            (21105, "unstrung_topaz_amulet", "Topaz amulet (u)", true),
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
    fn amulet_all_eleven_string_at_level_one_with_four_xp_and_atomic_full_bag_exchange() {
        let (mut defs, objects, stat) = fixture();
        let recipes = amulet_stringing(&objects, &stat, &defs).unwrap();
        assert_eq!(recipes.len(), 11);
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
            for (slot, id) in [(19, 1759), (27, recipe.second), (3, recipe.second)] {
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
            let job = Job {
                recipe,
                first_slot: 19,
                second_slot: 27,
            };
            let before = (inv.clone(), stats.clone());
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
            assert_eq!(stats.fine_xp(12), 50);
            assert_eq!(stats.current(12), 1);
            assert_eq!(inv.get(3), before.0.get(3));
            assert_eq!(
                inv.slots()
                    .iter()
                    .flatten()
                    .filter(|i| i.id == recipe.output)
                    .count(),
                1
            );
            assert!(inv.slots().iter().flatten().all(|i| i.id != 1759));
        }
        defs.items.get_mut(&21114).unwrap().members = false;
        assert!(amulet_stringing(&objects, &stat, &defs).is_err());
    }
    #[test]
    fn amulet_actual_held_registry_dispatch_rechecks_membership_and_selected_pairs() {
        use gielinor_world::{
            held_use::{Handler, Request, Scope, Type},
            simulation::{PlayerData, World, WorldEvent},
            CoordGrid,
        };
        let (defs, objects, stat) = fixture();
        let recipes = amulet_stringing(&objects, &stat, &defs).unwrap();
        for case in 0..5 {
            let recipe = recipes[if case == 2 { 5 } else { 0 }];
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
                    Handler::AssemblyTimed { recipe, ticks: 1 },
                )
                .unwrap();
            let mut data = PlayerData::new(
                1,
                "Stringer".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                Stats::default(),
            );
            data.xp_rate_hundreds = 125;
            data.inventory
                .restore_slot(
                    7,
                    Some(Item {
                        id: recipe.first,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            data.inventory
                .restore_slot(
                    25,
                    Some(Item {
                        id: recipe.second,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            data.inventory
                .restore_slot(
                    4,
                    Some(Item {
                        id: recipe.second,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            w.queue_login(1, data).unwrap();
            w.tick().unwrap();
            let a = w.actors().next().unwrap();
            w.set_membership(a, true).unwrap();
            let request = if case == 1 {
                Request {
                    selected_slot: 7,
                    selected_id: recipe.first,
                    target_slot: 25,
                    target_id: recipe.second,
                }
            } else {
                Request {
                    selected_slot: 25,
                    selected_id: recipe.second,
                    target_slot: 7,
                    target_id: recipe.first,
                }
            };
            w.prepare_held_use(a, request).unwrap();
            w.finish_held_use(a, request).unwrap();
            match case {
                2 => w.set_membership(a, false).unwrap(),
                3 => {
                    w.set_modal_open(a, true).unwrap();
                    w.set_modal_open(a, false).unwrap();
                }
                4 => {
                    w.data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(25, None)
                        .unwrap();
                }
                _ => {}
            }
            w.tick().unwrap();
            assert_eq!(w.data(a).unwrap().stats.fine_xp(12), 0);
            w.tick().unwrap();
            let expected = case < 2;
            assert_eq!(
                w.data(a).unwrap().stats.fine_xp(12),
                if expected { 50 } else { 0 }
            );
            assert_eq!(
                w.data(a).unwrap().inventory.get(4).unwrap().id,
                recipe.second
            );
            let traces = w
                .drain_events()
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
                assert_eq!(traces[0].committed_at - traces[0].queued_at, 1);
                assert_eq!(
                    (traces[0].job.first_slot, traces[0].job.second_slot),
                    (7, 25)
                );
            }
        }
    }
}
