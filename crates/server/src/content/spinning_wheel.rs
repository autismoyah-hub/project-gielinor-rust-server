//! Six classic spinning recipes at the attested ordinary spinning-wheel type.
use super::{Definitions, Symbols};
use gielinor_world::facility_crafting::{Config, Recipe};
pub fn spinning_wheel(
    objects: &Symbols,
    locations: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Config, String> {
    if stats.get("crafting") != Some(&12)
        || locations.get("spinningwheel") != Some(&14889)
        || sequences.get("human_spinningwheel") != Some(&894)
        || sequences.get("spinningwheel") != Some(&466)
    {
        return Err("Missing/aliased spinning wheel bindings".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    let loc = defs
        .locations
        .get(&14889)
        .ok_or("Missing ordinary spinning wheel")?;
    if loc.name != "Spinning wheel"
        || loc.ops[1].as_deref() != Some("Spin")
        || loc.collision.width != 1
        || loc.collision.length != 1
        || !loc.multi.is_empty()
        || loc.multi_default != -1
        || loc.multi_varbit != -1
        || loc.multi_varp != -1
    {
        return Err("Invalid ordinary spinning wheel".into());
    }
    for (symbol, id, name, member) in [
        ("wool", 1737, "Wool", false),
        ("ball_of_wool", 1759, "Ball of wool", false),
        ("flax", 1779, "Flax", true),
        ("bow_string", 1777, "Bow string", true),
        ("xbows_sinew", 9436, "Sinew", true),
        ("xbows_crossbow_string", 9438, "Crossbow string", true),
        ("oak_roots", 6043, "Oak roots", true),
        ("magic_roots", 6051, "Magic roots", true),
        ("magic_string", 6038, "Magic string", true),
        ("yak_hair", 10814, "Hair", true),
        ("rope", 954, "Rope", false),
    ] {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased spinning item {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing spinning item")?;
        if d.name != name
            || d.members != member
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid spinning item {id}"));
        }
    }
    Ok(Config {
        locations: [(14889, 2)].into(),
        recipes: std::sync::Arc::new(vec![
            Recipe {
                input: 1737,
                output: 1759,
                level: 1,
                xp_tenths: 25,
                members: false,
                label: "ball of wool",
            },
            Recipe {
                input: 1779,
                output: 1777,
                level: 10,
                xp_tenths: 150,
                members: true,
                label: "bow string",
            },
            Recipe {
                input: 9436,
                output: 9438,
                level: 10,
                xp_tenths: 150,
                members: true,
                label: "crossbow string (sinew)",
            },
            Recipe {
                input: 6043,
                output: 9438,
                level: 10,
                xp_tenths: 150,
                members: true,
                label: "crossbow string (oak roots)",
            },
            Recipe {
                input: 6051,
                output: 6038,
                level: 19,
                xp_tenths: 300,
                members: true,
                label: "magic string",
            },
            Recipe {
                input: 10814,
                output: 954,
                level: 30,
                xp_tenths: 250,
                members: true,
                label: "rope",
            },
        ]),
        ticks: 3,
        animation: 894,
        location_animation: Some(466),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        facility_crafting,
        inventory::{Inventory, Item, StackType},
        stats::Stats,
    };
    #[test]
    fn spinning_wheel_exact_classic_bindings_and_fractional_unary_credits_are_attested() {
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
        defs.locations.insert(
            14889,
            gielinor_cache::locations::LocationType::decode(b"\x02Spinning wheel\0\x1fSpin\0\0")
                .unwrap(),
        );
        let mut objects = Symbols::new();
        for (id, symbol, name, member) in [
            (1737, "wool", "Wool", false),
            (1759, "ball_of_wool", "Ball of wool", false),
            (1779, "flax", "Flax", true),
            (1777, "bow_string", "Bow string", true),
            (9436, "xbows_sinew", "Sinew", true),
            (9438, "xbows_crossbow_string", "Crossbow string", true),
            (6043, "oak_roots", "Oak roots", true),
            (6051, "magic_roots", "Magic roots", true),
            (6038, "magic_string", "Magic string", true),
            (10814, "yak_hair", "Hair", true),
            (954, "rope", "Rope", false),
        ] {
            objects.insert(symbol.into(), id);
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
        let locations = [("spinningwheel".into(), 14889)].into();
        let seq = [
            ("human_spinningwheel".into(), 894),
            ("spinningwheel".into(), 466),
        ]
        .into();
        let stat = [("crafting".into(), 12)].into();
        let config = spinning_wheel(&objects, &locations, &seq, &stat, &defs).unwrap();
        assert_eq!(config.recipes.len(), 6);
        assert_eq!(config.locations[&14889], 2);
        let cat = defs.inventory_catalog();
        for (recipe, fine) in config
            .recipes
            .iter()
            .copied()
            .zip([31, 187, 187, 187, 375, 312])
        {
            let mut inv = Inventory::new(1, StackType::Normal, false);
            inv.restore_slot(
                0,
                Some(Item {
                    id: recipe.input,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
            let mut stats = Stats::default();
            stats.set_base(12, recipe.level as i8);
            stats.set_current(12, recipe.level as i8);
            let (next, next_stats, advanced) =
                facility_crafting::prepare(recipe, 0, &inv, &cat, &stats, 125, true).unwrap();
            assert_eq!(next.get(0).unwrap().id, recipe.output);
            assert_eq!(next_stats.fine_xp(12), fine);
            assert!(advanced.is_some());
            if recipe.members {
                assert!(
                    facility_crafting::prepare(recipe, 0, &inv, &cat, &stats, 125, false).is_err()
                );
            }
        }
        defs.locations.get_mut(&14889).unwrap().ops[1] = None;
        assert!(spinning_wheel(&objects, &locations, &seq, &stat, &defs).is_err());
    }
}
