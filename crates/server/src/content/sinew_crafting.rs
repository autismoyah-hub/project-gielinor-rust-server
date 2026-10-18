//! Manual damaged-tail conversion; cooking and automatic production are separate.
use super::{Definitions, Symbols};
use gielinor_world::{grinding, tool_crafting::Recipe};

pub fn sinew_crafting(
    objects: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Recipe, String> {
    if stats.get("crafting") != Some(&12) {
        return Err("Missing/aliased Crafting stat".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    for (symbol, id, name, member) in [
        ("knife", 946, "Knife", false),
        ("damaged_ballista_rope", 19665, "Damaged monkey tail", true),
        ("xbows_sinew", 9436, "Sinew", true),
    ] {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased immediate tool item {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing immediate tool definition")?;
        if d.name != name
            || d.members != member
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid immediate tool item {id}"));
        }
    }
    if sequences.get("human_cutting_knife") != Some(&10937) {
        return Err("Missing/aliased knife-cutting sequence".into());
    }
    Ok(Recipe {
        exchange: grinding::Recipe {
            tool: 946,
            input: 19665,
            output: 9436,
            members: true,
            animation: Some(10937),
        },
        level: 10,
        xp_tenths: 150,
        ticks: 1,
        name: "sinew",
        action: "cut",
        failure: None,
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
    #[test]
    fn sinew_crafting_actual_reverse_held_action_retains_tool_and_manual_scope() {
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
        for (id, symbol, name, member) in [
            (946, "knife", "Knife", false),
            (19665, "damaged_ballista_rope", "Damaged monkey tail", true),
            (9436, "xbows_sinew", "Sinew", true),
        ] {
            objects.insert(symbol.into(), id);
            let mut raw = vec![2];
            raw.extend_from_slice(name.as_bytes());
            raw.push(0);
            if member {
                raw.push(16);
            }
            raw.push(0);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&raw).unwrap(),
            );
        }
        let sequences = [("human_cutting_knife".into(), 10937)].into();
        let stats = [("crafting".into(), 12)].into();
        let recipe = sinew_crafting(&objects, &sequences, &stats, &defs).unwrap();
        for member in [false, true] {
            let mut w = World::default();
            w.object_types = defs.inventory_catalog();
            w.collision.allocate(3200, 3200, 0);
            let mut stats = Stats::default();
            stats.set_base(3, 10);
            stats.set_current(3, 10);
            stats.set_base(12, 10);
            stats.set_current(12, 10);
            let mut data = PlayerData::new(
                1,
                "Tail crafter".into(),
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
            for (slot, id) in [(5, 946), (19, 19665), (21, 19665)] {
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
            w.set_membership(a, member).unwrap();
            for id in [946, 19665] {
                w.held_use_types.insert(id, Type { id, content: -1 });
            }
            w.held_use_handlers
                .register(Scope::Types, 946, 19665, Handler::ToolCrafting(recipe))
                .unwrap();
            let request = Request {
                selected_id: 19665,
                selected_slot: 19,
                target_id: 946,
                target_slot: 5,
            };
            w.prepare_held_use(a, request).unwrap();
            w.finish_held_use(a, request).unwrap();
            assert!(w.take_tool_crafting_request(a).unwrap().is_none());
            assert_eq!(w.data(a).unwrap().inventory.get(19).unwrap().id, 19665);
            for _ in 0..6 {
                w.tick().unwrap();
            }
            let d = w.data(a).unwrap();
            assert_eq!(
                d.inventory.get(19).unwrap().id,
                if member { 9436 } else { 19665 }
            );
            assert_eq!(d.stats.fine_xp(12), if member { 187 } else { 0 });
            assert_eq!(d.inventory.get(21).unwrap().id, 19665);
            assert_eq!(d.inventory.get(5).unwrap().id, 946);
            assert_eq!(d.inventory.free_space(), 0);
        }
        defs.items.get_mut(&9436).unwrap().cert.template = 799;
        assert!(sinew_crafting(&objects, &sequences, &stats, &defs).is_err());
    }
}
