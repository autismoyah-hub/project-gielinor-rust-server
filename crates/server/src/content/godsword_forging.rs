//! Two disjoint blade pieces per join. Three loose shards therefore require
//! two real joins, each awarding Wiki100XP rather than a one-step source shortcut.
use super::{Definitions, Symbols};
use gielinor_world::facility_crafting::{Config, Policy, Recipe, Tools};
use std::{collections::HashSet, sync::Arc};
const PARTS: [(&str, u32, &str, u8); 7] = [
    ("godwars_godsword_blade1", 11818, "Godsword shard 1", 1),
    ("godwars_godsword_blade2", 11820, "Godsword shard 2", 2),
    ("godwars_godsword_blade3", 11822, "Godsword shard 3", 4),
    (
        "godwars_godsword_blade1+2",
        11794,
        "Godsword shards 1 & 2",
        3,
    ),
    (
        "godwars_godsword_blade1+3",
        11796,
        "Godsword shards 1 & 3",
        5,
    ),
    (
        "godwars_godsword_blade2+3",
        11800,
        "Godsword shards 2 & 3",
        6,
    ),
    ("godwars_godsword_blade1+2+3", 11798, "Godsword blade", 7),
];
pub struct GodswordForging {
    pub facility: Config,
    pub policy: Policy,
    pub parts: [i32; 6],
}
pub fn godsword_forging(
    objects: &Symbols,
    locations: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    content: &Symbols,
    d: &Definitions,
    anvils: &HashSet<u32>,
) -> Result<GodswordForging, String> {
    if locations.get("anvil") != Some(&2097)
        || sequences.get("human_smithing") != Some(&898)
        || stats.get("smithing") != Some(&13)
        || content.get("smithing_anvil") != Some(&49)
    {
        return Err("Unattested blade forge bindings".into());
    }
    let stat = d.stats.get(&13).ok_or("Missing Smithing")?;
    if stat.name != "Smithing" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Unattested blade forge skill".into());
    }
    for (sym, id, name, slot, member) in PARTS
        .map(|(s, id, n, _)| (s, id, n, -1, true))
        .into_iter()
        .chain([
            ("hammer", 2347, "Hammer", -1, false),
            ("imcando_hammer", 25644, "Imcando hammer", 3, true),
            (
                "imcando_hammer_offhand",
                29775,
                "Imcando hammer (off-hand)",
                5,
                true,
            ),
        ])
    {
        let t = d.items.get(&id).ok_or("Missing forge material/tool")?;
        if objects.get(sym) != Some(&(id as i32))
            || t.name != name
            || t.members != member
            || t.wear != [slot, -1, -1]
            || t.stackable
            || t.dummy == 1
            || t.cert.template != 0
            || t.placeholder.template != 0
            || t.transform.template != 0
            || !t.objvars.is_empty()
        {
            return Err(format!("Unattested forge material/tool {id}"));
        }
    }
    let locations = anvils
        .iter()
        .filter_map(|&id| {
            let t = d.locations.get(&id)?;
            (t.content_group == 49
                && t.category == 772
                && t.name == "Anvil"
                && t.ops[0].as_deref() == Some("Smith")
                && t.collision.width == 1
                && t.collision.length == 1
                && t.multi.is_empty()
                && t.multi_default == -1
                && t.multi_varbit == -1
                && t.multi_varp == -1)
                .then_some((id, None))
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    if !locations.contains_key(&2097) {
        return Err("Missing ordinary surface anvil2097".into());
    }
    let mut recipes = Vec::new();
    for first in 0..6 {
        for second in first + 1..6 {
            let (_, a, _, left) = PARTS[first];
            let (_, b, _, right) = PARTS[second];
            if left & right != 0 {
                continue;
            }
            let (_, output, label, _) = PARTS
                .iter()
                .copied()
                .find(|p| p.3 == (left | right))
                .ok_or("Invalid disjoint blade union")?;
            recipes.push(Recipe {
                input: a as i32,
                input_count: 1,
                secondary: Some(b as i32),
                required_inventory_tools: &[],
                byproduct: None,
                output: output as i32,
                output_count: 1,
                level: 80,
                xp_tenths: 1000,
                members: true,
                first_ticks: 7,
                cycle_ticks: 7,
                failure: None,
                label,
            });
        }
    }
    if recipes.len() != 6 {
        return Err("Invalid blade join catalog".into());
    }
    Ok(GodswordForging {
        facility: Config {
            locations,
            recipes: Arc::new(recipes),
            action: "forge",
            empty_message: "You need another part of the Godsword blade to forge them together.",
            animation: 898,
            location_animation: None,
        },
        policy: Policy {
            skill: 13,
            tools: Some(Tools {
                carried: &[2347, 25644, 29775],
                worn: &[(3, 25644), (5, 29775)],
                message: "You need a hammer to work the blade pieces on an anvil.",
            }),
        },
        parts: PARTS[..6]
            .iter()
            .map(|p| p.1 as i32)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    pub(super) fn prepared() -> (
        Symbols,
        Symbols,
        Symbols,
        Symbols,
        Definitions,
        HashSet<u32>,
    ) {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut n = Symbols::new();
        for (sym, id, name, slot, member) in PARTS
            .map(|(s, id, n, _)| (s, id, n, -1, true))
            .into_iter()
            .chain([
                ("hammer", 2347, "Hammer", -1, false),
                ("imcando_hammer", 25644, "Imcando hammer", 3, true),
                (
                    "imcando_hammer_offhand",
                    29775,
                    "Imcando hammer (off-hand)",
                    5,
                    true,
                ),
            ])
        {
            let mut t = gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();
            t.name = name.into();
            t.members = member;
            t.wear = [slot, -1, -1];
            d.items.insert(id, t);
            n.insert(sym.into(), id as i32);
        }
        let mut stat = gielinor_cache::definitions::StatType::decode(b"\0").unwrap();
        stat.name = "Smithing".into();
        d.stats.insert(13, stat);
        let mut anvil = gielinor_cache::locations::LocationType::decode(b"\0").unwrap();
        anvil.name = "Anvil".into();
        anvil.content_group = 49;
        anvil.category = 772;
        anvil.ops[0] = Some("Smith".into());
        d.locations.insert(2097, anvil);
        (
            n,
            [("human_smithing".into(), 898)].into(),
            [("smithing".into(), 13)].into(),
            [("smithing_anvil".into(), 49)].into(),
            d,
            [2097].into(),
        )
    }
    #[test]
    fn blade_forge_factory_has_six_disjoint_nonidentity_joins_and_ten_attested_types() {
        let (n, s, t, c, d, a) = prepared();
        let f = godsword_forging(&n, &[("anvil".into(), 2097)].into(), &s, &t, &c, &d, &a).unwrap();
        assert_eq!(f.facility.recipes.len(), 6);
        for r in f.facility.recipes.iter() {
            assert_eq!(
                (r.level, r.xp_tenths, r.first_ticks, r.members),
                (80, 1000, 7, true)
            );
            assert_ne!(r.input, r.output);
            assert_ne!(r.secondary, Some(r.output));
        }
        assert_eq!(
            f.facility
                .recipes
                .iter()
                .filter(|r| r.output == 11798)
                .count(),
            3
        );
        assert_eq!(f.policy.skill, 13);
        assert_eq!(f.parts.len(), 6);
    }
    #[test]
    fn blade_forge_factory_refuses_material_tool_skill_sequence_and_surface_anvil_drift() {
        for id in [
            11818, 11820, 11822, 11794, 11796, 11800, 11798, 2347, 25644, 29775,
        ] {
            for change in 0..10 {
                let (mut n, s, t, c, mut d, a) = prepared();
                let item = d.items.get_mut(&id).unwrap();
                match change {
                    0 => {
                        n.retain(|_, v| *v != id as i32);
                    }
                    1 => item.name = "Other".into(),
                    2 => item.members = !item.members,
                    3 => item.wear = [9, -1, -1],
                    4 => item.stackable = true,
                    5 => item.dummy = 1,
                    6 => item.cert.template = 799,
                    7 => item.placeholder.template = 799,
                    8 => item.transform.template = 799,
                    _ => item.objvars.push(1),
                }
                assert!(
                    godsword_forging(&n, &[("anvil".into(), 2097)].into(), &s, &t, &c, &d, &a)
                        .is_err(),
                    "id{id}change{change}"
                );
            }
        }
        for change in 0..12 {
            let (n, mut s, mut t, mut c, mut d, mut a) = prepared();
            match change {
                0 => s.clear(),
                1 => t.clear(),
                2 => c.clear(),
                3 => d.stats.get_mut(&13).unwrap().name = "Other".into(),
                4 => d.stats.get_mut(&13).unwrap().unreleased = true,
                5 => a.clear(),
                6 => d.locations.get_mut(&2097).unwrap().name = "Other".into(),
                7 => d.locations.get_mut(&2097).unwrap().ops[0] = None,
                8 => d.locations.get_mut(&2097).unwrap().content_group = 50,
                9 => d.locations.get_mut(&2097).unwrap().collision.width = 2,
                10 => d.locations.get_mut(&2097).unwrap().multi_varbit = 1,
                _ => d.locations.get_mut(&2097).unwrap().category = -1,
            };
            assert!(
                godsword_forging(&n, &[("anvil".into(), 2097)].into(), &s, &t, &c, &d, &a).is_err(),
                "binding{change}"
            );
        }
    }
}

#[cfg(test)]
#[path = "godsword_forging_world_tests.rs"]
mod world_tests;
