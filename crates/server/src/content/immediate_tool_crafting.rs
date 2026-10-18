//! One-click, tool-preserving crafting from independently reviewed recipe facts.
use super::{Definitions, Symbols};
use gielinor_world::{
    grinding,
    tool_crafting::{Failure, Recipe},
};

pub fn immediate_tool_crafting(
    objects: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<Recipe>, String> {
    if stats.get("crafting") != Some(&12) {
        return Err("Missing/aliased Crafting stat".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    for (symbol, id, name, member) in [
        ("knife", 946, "Knife", false),
        ("dramen_branch", 771, "Dramen branch", true),
        ("dramen_staff", 772, "Dramen staff", true),
        ("chisel", 1755, "Chisel", false),
        ("limestone", 3211, "Limestone", true),
        ("limestonebrick", 3420, "Limestone brick", true),
        ("rock", 968, "Rock", true),
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
    for (name, id) in [
        ("human_cutting_knife", 10937),
        ("human_limestonecutting", 1309),
    ] {
        if sequences.get(name) != Some(&id) {
            return Err(format!("Missing/aliased immediate tool sequence {name}"));
        }
    }
    Ok(vec![
        Recipe {
            exchange: grinding::Recipe {
                tool: 946,
                input: 771,
                output: 772,
                members: true,
                animation: Some(10937),
            },
            level: 31,
            xp_tenths: 0,
            ticks: 0,
            name: "Dramen staff",
            action: "carve",
            output_count: 1,
            failure: None,
        },
        Recipe {
            exchange: grinding::Recipe {
                tool: 1755,
                input: 3211,
                output: 3420,
                members: true,
                animation: Some(1309),
            },
            level: 12,
            xp_tenths: 60,
            ticks: 0,
            name: "limestone brick",
            action: "cut",
            // Source curve reaches certainty at level 40, consistent with the Wiki.
            // Wiki failure XP (1.5) overrides the donor's omitted/default-zero reward.
            output_count: 1,
            failure: Some(Failure {
                output: 968,
                xp_tenths: 15,
                low: 137,
                high: 434,
                message: "You fail to cut the limestone into a brick.",
            }),
        },
    ])
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        inventory::{Inventory, Item, StackType},
        npc::RandomSource,
        stats::Stats,
        tool_crafting::Job,
    };
    struct Draw {
        value: u32,
        calls: usize,
    }
    impl RandomSource for Draw {
        fn below(&mut self, n: u32) -> u32 {
            assert_eq!(n, 256);
            self.calls += 1;
            self.value
        }
    }
    #[test]
    fn immediate_recipe_bindings_zero_xp_and_limestone_boundary_rewards_are_exact() {
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
            (771, "dramen_branch", "Dramen branch", true),
            (772, "dramen_staff", "Dramen staff", true),
            (1755, "chisel", "Chisel", false),
            (3211, "limestone", "Limestone", true),
            (3420, "limestonebrick", "Limestone brick", true),
            (968, "rock", "Rock", true),
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
        let seq = [
            ("human_cutting_knife".into(), 10937),
            ("human_limestonecutting".into(), 1309),
        ]
        .into();
        let stat = [("crafting".into(), 12)].into();
        let recipes = immediate_tool_crafting(&objects, &seq, &stat, &defs).unwrap();
        assert_eq!(recipes.len(), 2);
        let cat = defs.inventory_catalog();
        for (i, recipe) in recipes.into_iter().enumerate() {
            let mut inv = Inventory::new(2, StackType::Normal, false);
            for (slot, id) in [(0, recipe.exchange.tool), (1, recipe.exchange.input)] {
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
            let job = Job {
                recipe,
                tool_slot: 0,
                input_slot: 1,
            };
            let mut stats = Stats::default();
            stats.set_base(12, recipe.level as i8);
            stats.set_current(12, recipe.level as i8);
            if i == 0 {
                let mut rng = Draw {
                    value: 255,
                    calls: 0,
                };
                let (p, success) = job.resolve(&inv, &cat, &stats, 0, true, &mut rng).unwrap();
                assert!(success);
                assert!(p.advancement.is_none());
                assert_eq!(p.stats, stats);
                assert_eq!(rng.calls, 0);
                assert_eq!(p.inventory.get(1).unwrap().id, 772);
            } else {
                for (level, roll, success, id, fine) in [
                    (12, 170, true, 3420, 75),
                    (12, 171, false, 968, 18),
                    (39, 255, false, 968, 18),
                    (40, 255, true, 3420, 75),
                    (99, 255, true, 3420, 75),
                ] {
                    stats.set_current(12, level);
                    let mut rng = Draw {
                        value: roll,
                        calls: 0,
                    };
                    let (p, actual) = job
                        .resolve(&inv, &cat, &stats, 125, true, &mut rng)
                        .unwrap();
                    assert_eq!(actual, success);
                    assert_eq!(rng.calls, 1);
                    assert_eq!(p.inventory.get(1).unwrap().id, id);
                    assert_eq!(p.stats.fine_xp(12), fine);
                    assert!(p.advancement.is_some());
                    assert_eq!(p.inventory.get(0), inv.get(0));
                }
            }
            assert!(job.prepare(&inv, &cat, &stats, 125, false).is_err());
            stats.set_current(12, recipe.level as i8 - 1);
            assert!(job.prepare(&inv, &cat, &stats, 125, true).is_err());
        }
        defs.items.get_mut(&968).unwrap().placeholder.template = 14401;
        assert!(immediate_tool_crafting(&objects, &seq, &stat, &defs).is_err());
    }
}
