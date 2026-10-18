//! Bear, rat and rabbit adapters over the existing manual cooking transaction.
use super::{Definitions, Symbols};
use gielinor_world::{
    cooking::{Config, Recipe},
    food::Edible,
};
use std::collections::BTreeMap;

fn checked(
    objects: &Symbols,
    defs: &Definitions,
    symbol: &str,
    expected: i32,
    name: &str,
    member: bool,
    eat: bool,
) -> Result<i32, String> {
    let id = *objects
        .get(symbol)
        .ok_or_else(|| format!("Missing meat symbol {symbol}"))?;
    let d = u32::try_from(id)
        .ok()
        .and_then(|v| defs.items.get(&v))
        .ok_or("Missing meat definition")?;
    if id != expected
        || d.name != name
        || d.members != member
        || d.stackable
        || d.dummy == 1
        || d.cert.template != 0
        || d.placeholder.template != 0
        || d.transform.template != 0
        || (eat && d.inventory_ops[0].as_deref() != Some("Eat"))
    {
        return Err(format!("Unattested ordinary meat profile {symbol}"));
    }
    Ok(id)
}

/// The range dependency is already checked by the ordinary range adapter.
/// Stage both maps before publication; shared cooked/burnt outputs are intended.
pub fn basic_meat_cooking(
    objects: &Symbols,
    locations: &Symbols,
    defs: &Definitions,
    config: &mut Config,
) -> Result<(), String> {
    let range = u32::try_from(*locations.get("range").ok_or("Missing meat range")?)
        .map_err(|_| "Invalid meat range")?;
    if range != 26181 {
        return Err("Unattested meat range".into());
    }
    let mut ranged = config
        .surfaces
        .get(&range)
        .ok_or("Configure ordinary range before meat")?
        .recipes
        .clone();
    let mut fire = config.recipes.clone();
    let burnt = checked(
        objects,
        defs,
        "burnt_meat",
        2146,
        "Burnt meat",
        false,
        false,
    )?;
    let meat = checked(
        objects,
        defs,
        "cooked_meat",
        2142,
        "Cooked meat",
        false,
        true,
    )?;
    let rabbit = checked(
        objects,
        defs,
        "cooked_rabbit",
        3228,
        "Cooked rabbit",
        true,
        true,
    )?;
    for (symbol, id, name, member, cooked, fire_ticks) in [
        ("raw_bear_meat", 2136, "Raw bear meat", false, meat, 1),
        ("raw_rat_meat", 2134, "Raw rat meat", false, meat, 1),
        ("raw_rabbit", 3226, "Raw rabbit", true, rabbit, 4),
    ] {
        let raw = checked(objects, defs, symbol, id, name, member, false)?;
        if fire.contains_key(&raw) || ranged.contains_key(&raw) {
            return Err("Duplicate meat cooking input".into());
        }
        let recipe = Recipe {
            raw,
            cooked,
            burnt,
            level: 1,
            stop_burn: Some(34),
            low: 128,
            high: 512,
            xp: 30,
            fine_xp_override: None,
            ticks: fire_ticks,
        };
        fire.insert(raw, recipe);
        ranged.insert(raw, Recipe { ticks: 1, ..recipe });
    }
    config.recipes = fire;
    config.surfaces.get_mut(&range).unwrap().recipes = ranged;
    config.member_inputs.insert(3226);
    Ok(())
}

/// Cooked rabbit is ordinary member food; roasting and Regicide are separate.
pub fn rabbit_food(
    objects: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<i32, String> {
    let id = checked(
        objects,
        defs,
        "cooked_rabbit",
        3228,
        "Cooked rabbit",
        true,
        true,
    )?;
    if foods.contains_key(&id) {
        return Err("Duplicate cooked rabbit food".into());
    }
    foods.insert(
        id,
        Edible {
            id,
            heal: 5,
            delay: 3,
            replacement: None,
            message: "You eat the cooked rabbit.".into(),
        },
    );
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        cooking::Surface,
        inventory::{Catalog, Inventory, Item, StackType},
        npc::RandomSource,
        stats::Stats,
    };
    struct Draw(u32);
    impl RandomSource for Draw {
        fn below(&mut self, n: u32) -> u32 {
            assert_eq!(n, 256);
            self.0
        }
    }
    #[test]
    fn basic_meat_canonical_variants_selected_transactions_and_atomic_registration() {
        let mut objects = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (symbol, id, name, member, eat) in [
            ("raw_bear_meat", 2136, "Raw bear meat", false, false),
            ("raw_rat_meat", 2134, "Raw rat meat", false, false),
            ("raw_rabbit", 3226, "Raw rabbit", true, false),
            ("cooked_meat", 2142, "Cooked meat", false, true),
            ("cooked_rabbit", 3228, "Cooked rabbit", true, true),
            ("burnt_meat", 2146, "Burnt meat", false, false),
        ] {
            objects.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            d.name = name.into();
            d.members = member;
            if eat {
                d.inventory_ops[0] = Some("Eat".into());
            }
            defs.items.insert(id as u32, d);
        }
        let locations = Symbols::from([("range".into(), 26181)]);
        let empty = Config {
            no_burn_cape: None, member_inputs: Default::default(),
            surfaces: BTreeMap::from([(
                26181,
                Surface {
                    animation: 896,
                    recipes: BTreeMap::new(),
                },
            )]),
            fire: 26185,
            animation: 897,
            skill: 7,
            max_level: 99,
            recipes: BTreeMap::new(),
            tutorial: None,
        };
        let mut config = empty.clone();
        basic_meat_cooking(&objects, &locations, &defs, &mut config).unwrap();
        assert_eq!(config.member_inputs.into_iter().collect::<Vec<_>>(), [3226]);
        for (raw, cooked, fire_ticks) in [(2136, 2142, 1), (2134, 2142, 1), (3226, 3228, 4)] {
            for (recipes, ticks) in [
                (&config.recipes, fire_ticks),
                (&config.surfaces[&26181].recipes, 1),
            ] {
                let r = recipes[&raw];
                assert_eq!((r.cooked, r.burnt, r.ticks), (cooked, 2146, ticks));
                for (level, draw, output, xp) in [(34, 255, cooked, 375), (1, 255, 2146, 0)] {
                    let mut inv = Inventory::new(2, StackType::Normal, false);
                    for slot in 0..2 {
                        inv.restore_slot(
                            slot,
                            Some(Item {
                                id: raw,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    }
                    let mut stats = Stats::default();
                    stats.set_base(7, level);
                    stats.set_current(7, level);
                    r.complete(
                        1,
                        &mut inv,
                        &Catalog::default(),
                        &mut stats,
                        7,
                        99,
                        125,
                        &mut Draw(draw),
                    )
                    .unwrap();
                    assert_eq!(inv.get(0).unwrap().id, raw);
                    assert_eq!(inv.get(1).unwrap().id, output);
                    assert_eq!(stats.fine_xp(7), xp);
                }
            }
        }
        let mut foods = BTreeMap::new();
        let id = rabbit_food(&objects, &defs, &mut foods).unwrap();
        for (current, after) in [(1, 6), (9, 10), (15, 15)] {
            let mut inv = Inventory::new(2, StackType::Normal, false);
            for slot in 0..2 {
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
            stats.set_base(3, 10);
            stats.set_current(3, current);
            gielinor_world::food::eat_slot(
                &foods[&id],
                1,
                &mut inv,
                &mut stats,
                &Default::default(),
                3,
                829,
                |_| {},
            )
            .unwrap();
            assert_eq!(stats.current(3), after);
            assert_eq!(stats.fine_xp(3), 0);
            assert_eq!(inv.get(0).unwrap().id, id);
            assert!(inv.get(1).is_none());
        }
        let mut config = empty.clone();
        defs.items.get_mut(&3226).unwrap().cert.template = 799;
        assert!(basic_meat_cooking(&objects, &locations, &defs, &mut config).is_err());
        assert!(config.recipes.is_empty());
        assert!(config.surfaces[&26181].recipes.is_empty());
        assert!(config.member_inputs.is_empty());
        defs.items.get_mut(&3228).unwrap().inventory_ops[0] = None;
        foods.clear();
        assert!(rabbit_food(&objects, &defs, &mut foods).is_err());
        assert!(foods.is_empty());
    }
}
