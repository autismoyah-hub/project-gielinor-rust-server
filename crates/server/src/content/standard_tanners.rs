//! Standard cowhide fees. Guild entry, dialogue and regional prices are separate.
use super::{Definitions, Symbols};
use gielinor_world::tanning::Recipe;
pub fn standard_tanners(
    objects: &Symbols,
    npcs: &Symbols,
    defs: &Definitions,
) -> Result<std::collections::BTreeMap<u16, Vec<Recipe>>, String> {
    for (symbol, id, name, stack) in [
        ("cow_hide", 1739, "Cowhide", false),
        ("leather", 1741, "Leather", false),
        ("hard_leather", 1743, "Hard leather", false),
        ("coins", 995, "Coins", true),
    ] {
        let item = defs
            .items
            .get(&id)
            .ok_or_else(|| format!("Missing tanning item {id}"))?;
        if objects.get(symbol) != Some(&i32::try_from(id).map_err(|_| "Invalid item ID")?)
            || item.name != name
            || item.members
            || item.stackable != stack
            || item.dummy == 1
            || item.cert.template != 0
            || item.placeholder.template != 0
            || item.transform.template != 0
        {
            return Err(format!("Invalid tanning item {id}"));
        }
    }
    let mut profiles = std::collections::BTreeMap::new();
    for (symbol, id, name) in [("ellis_tanner", 3231, "Ellis"), ("tanner", 5809, "Tanner")] {
        let npc = defs
            .npcs
            .get(&id)
            .ok_or_else(|| format!("Missing tanner {id}"))?;
        if npcs.get(symbol) != Some(&i32::try_from(id).map_err(|_| "Invalid NPC symbol ID")?)
            || npc.name != name
            || npc.size != 1
            || npc.ops[2].as_deref() != Some("Trade")
            || !npc.multi.is_empty()
            || npc.multi_default.is_some()
            || npc.multi_varbit.is_some()
            || npc.multi_varp.is_some()
        {
            return Err(format!("Invalid static tanner {id}"));
        }
        profiles.insert(
            u16::try_from(id).map_err(|_| "Invalid NPC protocol ID")?,
            vec![
                Recipe {
                    input: 1739,
                    output: 1741,
                    cost: 1,
                    members: false,
                },
                Recipe {
                    input: 1739,
                    output: 1743,
                    cost: 3,
                    members: false,
                },
            ],
        );
    }
    Ok(profiles)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tanning_profiles_validate_real_trade_and_both_cowhide_choices() {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (id, name, stack) in [
            (1739, "Cowhide", false),
            (1741, "Leather", false),
            (1743, "Hard leather", false),
            (995, "Coins", true),
        ] {
            let mut raw = vec![2];
            raw.extend(name.bytes());
            raw.push(0);
            if stack {
                raw.push(11);
            }
            raw.push(0);
            d.items.insert(
                id,
                gielinor_cache::definitions::ItemType::decode(&raw).unwrap(),
            );
        }
        for (id, name) in [(3231, "Ellis"), (5809, "Tanner")] {
            let mut raw = vec![2];
            raw.extend(name.bytes());
            raw.extend(b"\0\x20Trade\0\0");
            d.npcs
                .insert(id, gielinor_cache::npcs::NpcType::decode(&raw).unwrap());
        }
        let o = [
            ("cow_hide".into(), 1739),
            ("leather".into(), 1741),
            ("hard_leather".into(), 1743),
            ("coins".into(), 995),
        ]
        .into();
        let n = [("ellis_tanner".into(), 3231), ("tanner".into(), 5809)].into();
        let rows = standard_tanners(&o, &n, &d).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[&3231][1].cost, 3);
        d.npcs.get_mut(&3231).unwrap().ops[2] = None;
        assert!(standard_tanners(&o, &n, &d).is_err());
    }
}
