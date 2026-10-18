//! Wiki-backed Cooking gauntlet curves for already registered ordinary fish recipes.
use super::{Definitions, Symbols};
use gielinor_world::cooking::{Config, GauntletProfile, Gauntlets};
const ITEMS: [(&str, i32, &str, bool); 16] = [
    ("gauntlets_of_cooking", 775, "Cooking gauntlets", true),
    ("raw_lobster", 377, "Raw lobster", false),
    ("lobster", 379, "Lobster", false),
    ("burnt_lobster", 381, "Burnt lobster", false),
    ("raw_swordfish", 371, "Raw swordfish", false),
    ("swordfish", 373, "Swordfish", false),
    ("burnt_swordfish", 375, "Burnt swordfish", false),
    ("raw_monkfish", 7944, "Raw monkfish", true),
    ("monkfish", 7946, "Monkfish", true),
    ("burnt_monkfish", 7948, "Burnt monkfish", true),
    ("raw_shark", 383, "Raw shark", true),
    ("shark", 385, "Shark", true),
    ("burnt_shark", 387, "Burnt shark", true),
    ("raw_anglerfish", 13439, "Raw anglerfish", true),
    ("anglerfish", 13441, "Anglerfish", true),
    ("burnt_anglerfish", 13443, "Burnt anglerfish", true),
];
const CURVES: [(i32, i32, i32, u8, u8, i32, i32); 5] = [
    (377, 379, 381, 40, 64, 55, 368),
    (371, 373, 375, 45, 80, 30, 310),
    (7944, 7946, 7948, 62, 86, 24, 290),
    (383, 385, 387, 80, 94, 15, 270),
    (13439, 13441, 13443, 84, 97, 12, 260),
];
pub fn cooking_gauntlets(
    objects: &Symbols,
    locations: &Symbols,
    defs: &Definitions,
    config: &Config,
) -> Result<Gauntlets, String> {
    if locations.get("fire") != Some(&26185)
        || locations.get("range") != Some(&26181)
        || config.fire != 26185
        || config.skill != 7
    {
        return Err("Invalid gauntlet cooking surfaces".into());
    }
    for &(symbol, id, name, members) in &ITEMS {
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing gauntlet cooking item")?;
        if objects.get(symbol) != Some(&id)
            || d.name != name
            || d.members != members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid gauntlet cooking item {id}"));
        }
        if id == 775 && (d.wear != [9, -1, -1] || d.inventory_ops[1].as_deref() != Some("Wear")) {
            return Err("Invalid cooking gauntlets wear".into());
        }
    }
    let mut profiles = std::collections::BTreeMap::new();
    for (raw, cooked, burnt, level, stop_burn, low, high) in CURVES {
        for surface in [26185, 26181] {
            let p = config
                .recipe(surface, raw)
                .ok_or("Configure all five ordinary fish before gauntlets")?;
            if (p.raw, p.cooked, p.burnt, p.level) != (raw, cooked, burnt, level) {
                return Err("Conflicting cooking gauntlet recipe identity".into());
            }
        }
        profiles.insert(
            raw,
            GauntletProfile {
                cooked,
                burnt,
                level,
                stop_burn,
                low,
                high,
            },
        );
    }
    Ok(Gauntlets {
        item: 775,
        surfaces: [26185, 26181].into(),
        profiles,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cooking_gauntlets_bind_five_existing_exact_recipes_and_reject_late_invalid_catalog() {
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
        for &(sym, id, name, members) in &ITEMS {
            let mut t = gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();
            t.name = name.into();
            t.members = members;
            if id == 775 {
                t.wear = [9, -1, -1];
                t.inventory_ops[1] = Some("Wear".into());
            }
            d.items.insert(id as u32, t);
            objects.insert(sym.into(), id);
        }
        let recipes = CURVES
            .into_iter()
            .map(|(raw, cooked, burnt, level, ..)| {
                (
                    raw,
                    gielinor_world::cooking::Recipe {
                        raw,
                        cooked,
                        burnt,
                        level,
                        stop_burn: None,
                        low: 1,
                        high: 200,
                        xp: 120,
                        fine_xp_override: None,
                        ticks: 4,
                    },
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        let c = Config {
            gauntlets: None,
            no_burn_cape: None,
            member_inputs: Default::default(),
            surfaces: [(
                26181,
                gielinor_world::cooking::Surface {
                    animation: 897,
                    recipes: recipes.clone(),
                },
            )]
            .into(),
            fire: 26185,
            animation: 897,
            skill: 7,
            max_level: 99,
            recipes,
            tutorial: None,
        };
        let locations = [("fire".into(), 26185), ("range".into(), 26181)].into();
        let rule = cooking_gauntlets(&objects, &locations, &d, &c).unwrap();
        assert_eq!(rule.profiles.len(), 5);
        assert_eq!(
            (
                rule.profiles[&377].low,
                rule.profiles[&377].high,
                rule.profiles[&377].stop_burn
            ),
            (55, 368, 64)
        );
        d.items.get_mut(&13443).unwrap().transform.template = 1;
        assert!(cooking_gauntlets(&objects, &locations, &d, &c).is_err());
        d.items.get_mut(&13443).unwrap().transform.template = 0;
        let mut bad = c.clone();
        bad.surfaces
            .get_mut(&26181)
            .unwrap()
            .recipes
            .get_mut(&383)
            .unwrap()
            .level = 79;
        assert!(cooking_gauntlets(&objects, &locations, &d, &bad).is_err());
    }
}
