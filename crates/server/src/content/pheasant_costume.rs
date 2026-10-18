//! Thread-free costume-needle route; ordinary needle needs an attested thread counter.
use super::{Definitions, Symbols};
use gielinor_world::{grinding, tool_crafting::Recipe};
pub fn pheasant_costume(
    objects: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<Recipe>, String> {
    if stats.get("crafting") != Some(&12)
        || sequences.get("human_pheasant_feathers_crafting") != Some(&10534)
    {
        return Err("Missing pheasant Crafting/animation aliases".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    let check = |symbol: &str, id: i32, name: &str, stackable: bool| -> Result<(), String> {
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing pheasant definition")?;
        if objects.get(symbol) != Some(&id)
            || d.name != name
            || !d.members
            || d.stackable != stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid pheasant item {id}"));
        }
        Ok(())
    };
    check("costumeneedle", 29920, "Costume needle", false)?;
    check(
        "forestry_pheasant_feathers",
        28624,
        "Pheasant tail feathers",
        true,
    )?;
    let mut out = Vec::new();
    for (symbol, id, name) in [
        ("forestry_pheasant_boots", 28618, "Pheasant boots"),
        ("forestry_pheasant_hat", 28620, "Pheasant hat"),
        ("forestry_pheasant_legs", 28622, "Pheasant legs"),
        ("forestry_pheasant_cape", 28616, "Pheasant cape"),
    ] {
        check(symbol, id, name, false)?;
        if defs.items[&(id as u32)].inventory_ops[1].as_deref() != Some("Wear") {
            return Err("Invalid pheasant wearable output".into());
        }
        out.push(Recipe {
            skill: 12,
            exchange: grinding::Recipe {
                tool: 29920,
                input: 28624,
                output: id,
                members: true,
                animation: Some(10534),
            },
            level: 2,
            xp_tenths: 150,
            output_count: 1,
            ticks: 2,
            name,
            action: "make",
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
        inventory::Item,
        simulation::{PlayerData, World},
        stats::Stats,
        CoordGrid,
    };
    fn defs() -> (Definitions, Symbols, Symbols, Symbols) {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut objects = Symbols::new();
        for (name, id, label, stackable) in [
            ("costumeneedle", 29920, "Costume needle", false),
            (
                "forestry_pheasant_feathers",
                28624,
                "Pheasant tail feathers",
                true,
            ),
            ("forestry_pheasant_boots", 28618, "Pheasant boots", false),
            ("forestry_pheasant_hat", 28620, "Pheasant hat", false),
            ("forestry_pheasant_legs", 28622, "Pheasant legs", false),
            ("forestry_pheasant_cape", 28616, "Pheasant cape", false),
        ] {
            objects.insert(name.into(), id);
            let mut item = gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();
            item.name = label.into();
            item.members = true;
            item.stackable = stackable;
            if !stackable && id != 29920 {
                item.inventory_ops[1] = Some("Wear".into());
            }
            d.items.insert(id as u32, item);
        }
        d.stats.insert(
            12,
            gielinor_cache::definitions::StatType::decode(b"\x01Crafting\0\0").unwrap(),
        );
        (
            d,
            objects,
            [("human_pheasant_feathers_crafting".into(), 10534)].into(),
            [("crafting".into(), 12)].into(),
        )
    }
    #[test]
    fn pheasant_costume_binds_four_member_outputs_and_rejects_late_stackable() {
        let (mut d, o, s, t) = defs();
        let recipes = pheasant_costume(&o, &s, &t, &d).unwrap();
        assert_eq!(
            recipes
                .iter()
                .map(|r| r.exchange.output)
                .collect::<Vec<_>>(),
            vec![28618, 28620, 28622, 28616]
        );
        assert!(recipes
            .iter()
            .all(|r| r.level == 2 && r.xp_tenths == 150 && r.ticks == 2 && r.exchange.members));
        d.items.get_mut(&28616).unwrap().stackable = true;
        assert!(pheasant_costume(&o, &s, &t, &d).is_err());
    }
    #[test]
    fn pheasant_costume_actual_selected_stack_two_then_three_ticks_all_choices_both_orders() {
        let (d, o, s, t) = defs();
        let recipes = std::sync::Arc::new(pheasant_costume(&o, &s, &t, &d).unwrap());
        for index in 0..4 {
            for reverse in [false, true] {
                let mut w = World::default();
                w.object_types.stackable.insert(28624);
                let mut stats = Stats::default();
                stats.set_base(12, 2);
                stats.set_current(12, 2);
                let mut data = PlayerData::new(1, "Pheasant".into(), CoordGrid::ZERO, stats);
                data.xp_rate_hundreds = 100;
                data.inventory
                    .restore_slot(
                        4,
                        Some(Item {
                            id: 29920,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                data.inventory
                    .restore_slot(
                        17,
                        Some(Item {
                            id: 28624,
                            count: 30,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                w.queue_login(1, data).unwrap();
                w.tick().unwrap();
                w.tick().unwrap();
                let actor = w.actors().next().unwrap();
                w.set_membership(actor, true).unwrap();
                for id in [29920, 28624] {
                    w.held_use_types.insert(id, Type { id, content: -1 });
                }
                w.held_use_handlers
                    .register(
                        Scope::Types,
                        29920,
                        28624,
                        Handler::ToolCraftingCountedChoices {
                            recipes: recipes.clone(),
                            cycle_ticks: 3,
                            input_count: 15,
                        },
                    )
                    .unwrap();
                let held = if reverse {
                    Request {
                        selected_slot: 17,
                        selected_id: 28624,
                        target_slot: 4,
                        target_id: 29920,
                    }
                } else {
                    Request {
                        selected_slot: 4,
                        selected_id: 29920,
                        target_slot: 17,
                        target_id: 28624,
                    }
                };
                w.finish_held_use(actor, held).unwrap();
                let request = w.take_tool_crafting_request(actor).unwrap().unwrap();
                assert_eq!(request.available(&w.data(actor).unwrap().inventory), 2);
                let start = w.clock.0;
                w.start_tool_crafting_choice(actor, request, index, 2)
                    .unwrap();
                let mut commits = Vec::new();
                let output = recipes[index].exchange.output;
                let mut old = 0;
                for _ in 0..8 {
                    w.tick().unwrap();
                    let now = w
                        .data(actor)
                        .unwrap()
                        .inventory
                        .slots()
                        .iter()
                        .flatten()
                        .filter(|i| i.id == output)
                        .count();
                    if now != old {
                        commits.push(w.clock.0 - 1);
                    }
                    old = now;
                }
                let data = w.data(actor).unwrap();
                assert_eq!(commits, vec![start + 2, start + 5]);
                assert_eq!(old, 2);
                assert_eq!(data.stats.fine_xp(12), 300);
                assert!(data.inventory.get(17).is_none());
                assert_eq!(data.inventory.get(4).unwrap().id, 29920);
            }
        }
    }
}
