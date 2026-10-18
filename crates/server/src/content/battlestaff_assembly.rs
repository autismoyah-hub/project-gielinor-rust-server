//! Manual elemental-orb attachment using shared precise assembly transactions.
use super::{Definitions, Symbols};
use gielinor_world::assembly::{PreciseRecipe, Recipe};
pub fn battlestaff_assembly(
    objects: &Symbols,
    sequences: &Symbols,
    spots: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<PreciseRecipe>, String> {
    if stats.get("crafting") != Some(&12) {
        return Err("Missing/aliased Crafting stat".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    let check = |symbol: &str, id: i32, name: &str| -> Result<(), String> {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased battlestaff item {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing battlestaff definition")?;
        if d.name != name
            || !d.members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid battlestaff item {id}"));
        }
        Ok(())
    };
    check("battlestaff", 1391, "Battlestaff")?;
    if sequences.get("human_battlestaff_crafting") != Some(&7531) {
        return Err("Missing/aliased battlestaff sequence".into());
    }
    let mut out = Vec::new();
    for (
        orb,
        orb_symbol,
        orb_name,
        output,
        output_symbol,
        name,
        level,
        xp_tenths,
        spot_symbol,
        spot,
    ) in [
        (
            571,
            "water_orb",
            "Water orb",
            1395,
            "water_battlestaff",
            "Water battlestaff",
            54,
            1000,
            "battlestaff_water_crafting_spotanim",
            1370,
        ),
        (
            575,
            "earth_orb",
            "Earth orb",
            1399,
            "earth_battlestaff",
            "Earth battlestaff",
            58,
            1125,
            "battlestaff_earth_crafting_spotanim",
            1371,
        ),
        (
            569,
            "fire_orb",
            "Fire orb",
            1393,
            "fire_battlestaff",
            "Fire battlestaff",
            62,
            1250,
            "battlestaff_fire_crafting_spotanim",
            1372,
        ),
        (
            573,
            "air_orb",
            "Air orb",
            1397,
            "air_battlestaff",
            "Air battlestaff",
            66,
            1375,
            "battlestaff_air_crafting_spotanim",
            306,
        ),
    ] {
        check(orb_symbol, orb, orb_name)?;
        check(output_symbol, output, name)?;
        if spots.get(spot_symbol) != Some(&spot) {
            return Err(format!("Missing/aliased battlestaff spot {spot_symbol}"));
        }
        out.push(PreciseRecipe {
            recipe: Recipe {
                first: 1391,
                second: orb,
                output,
                skill: 12,
                level,
                base_xp: 0,
                members: true,
            },
            xp_tenths,
            ticks: 2,
            animation: Some(7531),
            spot: Some(spot as u16),
        });
    }
    Ok(out)
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
    #[test]
    fn battlestaff_assembly_actual_dispatch_preserves_fractional_xp_and_manual_selected_pairs() {
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
        for (id, symbol, name) in [
            (1391, "battlestaff", "Battlestaff"),
            (571, "water_orb", "Water orb"),
            (575, "earth_orb", "Earth orb"),
            (569, "fire_orb", "Fire orb"),
            (573, "air_orb", "Air orb"),
            (1395, "water_battlestaff", "Water battlestaff"),
            (1399, "earth_battlestaff", "Earth battlestaff"),
            (1393, "fire_battlestaff", "Fire battlestaff"),
            (1397, "air_battlestaff", "Air battlestaff"),
        ] {
            objects.insert(symbol.into(), id);
            let mut raw = vec![2];
            raw.extend_from_slice(name.as_bytes());
            raw.extend_from_slice(&[0, 16, 0]);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&raw).unwrap(),
            );
        }
        let seq = [("human_battlestaff_crafting".into(), 7531)].into();
        let stat = [("crafting".into(), 12)].into();
        let spots = [
            ("battlestaff_water_crafting_spotanim".into(), 1370),
            ("battlestaff_earth_crafting_spotanim".into(), 1371),
            ("battlestaff_fire_crafting_spotanim".into(), 1372),
            ("battlestaff_air_crafting_spotanim".into(), 306),
        ]
        .into();
        let recipes = battlestaff_assembly(&objects, &seq, &spots, &stat, &defs).unwrap();
        assert_eq!(recipes.len(), 4);
        for (r, fine) in recipes.into_iter().zip([1250, 1406, 1562, 1718]) {
            let mut w = World::default();
            w.object_types = defs.inventory_catalog();
            w.collision.allocate(3200, 3200, 0);
            let mut stats = Stats::default();
            stats.set_base(3, 10);
            stats.set_current(3, 10);
            stats.set_base(12, r.recipe.level as i8);
            stats.set_current(12, r.recipe.level as i8);
            let mut data = PlayerData::new(
                1,
                "Orb crafter".into(),
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
            for (slot, id) in [
                (5, 1391),
                (19, r.recipe.second),
                (21, r.recipe.second),
                (23, 1391),
            ] {
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
            for id in [1391, r.recipe.second] {
                w.held_use_types.insert(id, Type { id, content: -1 });
            }
            w.held_use_handlers
                .register(
                    Scope::Types,
                    1391,
                    r.recipe.second,
                    Handler::AssemblyPrecise(r),
                )
                .unwrap();
            let held = Request {
                selected_slot: 19,
                selected_id: r.recipe.second,
                target_slot: 5,
                target_id: 1391,
            };
            w.prepare_held_use(a, held).unwrap();
            w.finish_held_use(a, held).unwrap();
            assert!(w.take_tool_crafting_request(a).unwrap().is_none());
            w.tick().unwrap();
            let events = w.drain_events();
            assert!(events
                .iter()
                .any(|e| matches!(e, WorldEvent::PlayerCombatAnimation { sequence: 7531, .. })));
            assert!(events.iter().any(
                |e| matches!(e,WorldEvent::PlayerSpotAnimation{spot,..}if Some(*spot)==r.spot)
            ));
            assert_eq!(w.data(a).unwrap().stats.fine_xp(12), 0);
            w.tick().unwrap();
            assert_eq!(w.data(a).unwrap().stats.fine_xp(12), 0);
            w.tick().unwrap();
            let d = w.data(a).unwrap();
            assert_eq!(d.stats.fine_xp(12), fine);
            assert_eq!(
                d.inventory
                    .slots()
                    .iter()
                    .flatten()
                    .filter(|i| i.id == r.recipe.output)
                    .count(),
                1
            );
            assert_eq!(d.inventory.get(21).unwrap().id, r.recipe.second);
            assert_eq!(d.inventory.get(23).unwrap().id, 1391);
            assert_eq!(d.inventory.free_space(), 1);
            for _ in 0..5 {
                w.tick().unwrap();
            }
            assert_eq!(w.data(a).unwrap().stats.fine_xp(12), fine);
        }
        defs.items.get_mut(&1397).unwrap().transform.template = 1;
        assert!(battlestaff_assembly(&objects, &seq, &spots, &stat, &defs).is_err());
    }
}
