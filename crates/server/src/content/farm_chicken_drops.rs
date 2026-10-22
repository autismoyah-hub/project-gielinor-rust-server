//! Additional attested ordinary farm variants, retaining the existing chicken

//! guaranteed/weighted death kernel. No tutorial/misc/clue/account expansion.

use super::{Definitions, Symbols};

use gielinor_world::{
    npc::{BlockWalk, Restrict},
    npc_death::{Chicken, Config, Definition},
};

use std::collections::BTreeMap;

const FARM: [(&str, u16); 7] = [
    ("farm_chicken_brown", 2804),
    ("farm_chicken_brown_indoors", 10497),
    ("farm_chicken_brown_outdoors", 10494),
    ("farm_chicken_dark_brown", 2805),
    ("farm_chicken_dark_brown_outdoors", 10495),
    ("farm_chicken_tan", 2806),
    ("farm_chicken_tan_outdoors", 10496),
];

pub struct FarmChickenDrops {
    pub drops: BTreeMap<u16, Chicken>,
    pub deaths: BTreeMap<u16, Definition>,
}
pub fn farm_chicken_drops(
    npcs: &Symbols,
    items: &Symbols,
    sequences: &Symbols,
    d: &Definitions,
    death: &Config,
) -> Result<FarmChickenDrops, String> {
    if death.bones != 526
        || [1173, 1174].into_iter().any(|id| {
            !death
                .chickens
                .get(&id)
                .is_some_and(|p| p.raw == 2138 && p.feather == 314)
        })
    {
        return Err("Missing attested base chicken drop kernel".into());
    }

    for (symbol, id, name, stack) in [
        ("bones", 526, "Bones", false),
        ("raw_chicken", 2138, "Raw chicken", false),
        ("feather", 314, "Feather", true),
    ] {
        let t = d.items.get(&id).ok_or("Missing chicken drop material")?;

        if items.get(symbol) != Some(&(id as i32))
            || t.name != name
            || t.stackable != stack
            || t.members
            || t.wear != [-1, -1, -1]
            || t.dummy == 1
            || t.cert.template != 0
            || t.placeholder.template != 0
            || t.transform.template != 0
            || !t.objvars.is_empty()
        {
            return Err("Unattested ordinary chicken product".into());
        }
    }

    // New farm definitions omit death_anim and resolve to generic human_death836.
    // Their attested ready/walk rig matches the two baseline chickens. Reuse that
    // existing family lifecycle explicitly; never extend this inference to misc/tutorial.
    let rig = [5386, 65535, 5385, 5385, 65535, 65535, 65535];
    if sequences.get("lore_chicken_ready") != Some(&5386)
        || sequences.get("lore_chicken_walk") != Some(&5385)
        || sequences.get("lore_chicken_death") != Some(&5389)
    {
        return Err("Unattested chicken family sequences".into());
    }
    let baseline = *death
        .definitions
        .get(&1173)
        .ok_or("Missing baseline chicken death")?;
    if (baseline.animation, baseline.animation_ticks, baseline.sound) != (5389, 4, Some(356)) {
        return Err("Unattested baseline chicken death".into());
    }
    for id in [1173, 1174] {
        let t = d.npcs.get(&id).ok_or("Missing baseline chicken rig")?;
        let end = death
            .definitions
            .get(&(id as u16))
            .ok_or("Missing baseline chicken death")?;
        if t.name != "Chicken"
            || t.animations != rig
            || t.params.get(&65502) != Some(&gielinor_cache::definitions::Param::Integer(5389))
            || (end.animation, end.animation_ticks, end.sound)
                != (baseline.animation, baseline.animation_ticks, baseline.sound)
        {
            return Err("Unattested baseline chicken rig/death".into());
        }
    }
    let mut out = BTreeMap::new();
    let mut deaths = BTreeMap::new();

    for (symbol, id) in FARM {
        let t = d
            .npcs
            .get(&u32::from(id))
            .ok_or("Missing farm chicken definition")?;

        if npcs.get(symbol) != Some(&i32::from(id))
            || t.name != "Chicken"
            || t.size != 1
            || t.stats != [1, 1, 1, 3, 1, 1]
            || t.content_group != -1
            || t.timer != -1
            || t.ops.iter().map(Option::as_deref).collect::<Vec<_>>()
                != [None, Some("Attack"), None, None, None]
            || t.restrict != Restrict::Normal
            || !matches!(t.block_walk, BlockWalk::Npc)
            || t.default_mode != 1
            || t.wander_range != 5
            || !t.multi.is_empty()
            || t.multi_varp.is_some()
            || t.multi_varbit.is_some()
            || t.multi_default.is_some()
        {
            return Err("Unattested ordinary farm chicken profile".into());
        }

        let observed = death
            .definitions
            .get(&id)
            .ok_or("Missing farm chicken death")?;
        let supported = match t.params.get(&65502) {
            None => {
                (observed.animation, observed.animation_ticks, observed.sound) == (836, 2, None)
            }
            Some(gielinor_cache::definitions::Param::Integer(5389)) => {
                (observed.animation, observed.animation_ticks, observed.sound)
                    == (5389, 4, Some(356))
            }
            _ => false,
        };
        if death.chickens.contains_key(&id) || t.animations != rig || !supported {
            return Err("Conflicting or unattested farm chicken rig/death".into());
        }
        deaths.insert(id, baseline);

        out.insert(
            id,
            Chicken {
                raw: 2138,
                feather: 314,
            },
        );
    }

    Ok(FarmChickenDrops { drops: out, deaths })
}

#[cfg(test)]

mod tests {

    use super::*;

    fn sequences() -> Symbols {
        [
            ("lore_chicken_ready".into(), 5386),
            ("lore_chicken_walk".into(), 5385),
            ("lore_chicken_death".into(), 5389),
        ]
        .into()
    }
    fn fixture() -> (Symbols, Symbols, Definitions, Config) {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };

        for (_, id) in FARM {
            let mut t = gielinor_cache::npcs::NpcType::decode(&[0]).unwrap();
            t.name = "Chicken".into();
            t.stats = [1, 1, 1, 3, 1, 1];
            t.ops[1] = Some("Attack".into());
            t.animations = [5386, 65535, 5385, 5385, 65535, 65535, 65535];
            t.default_mode = 1;
            t.wander_range = 5;
            t.block_walk = BlockWalk::Npc;
            t.restrict = Restrict::Normal;
            d.npcs.insert(id.into(), t);
        }

        for id in [1173, 1174] {
            let mut t = gielinor_cache::npcs::NpcType::decode(&[0]).unwrap();
            t.name = "Chicken".into();
            t.animations = [5386, 65535, 5385, 5385, 65535, 65535, 65535];
            t.params
                .insert(65502, gielinor_cache::definitions::Param::Integer(5389));
            d.npcs.insert(id, t);
        }
        for (id, name, stack) in [
            (526, "Bones", false),
            (2138, "Raw chicken", false),
            (314, "Feather", true),
        ] {
            let mut t = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            t.name = name.into();
            t.stackable = stack;
            d.items.insert(id, t);
        }

        let death = Config {
            definitions: FARM
                .into_iter()
                .map(|(_, id)| {
                    (
                        id,
                        Definition {
                            animation: 836,
                            animation_ticks: 2,
                            sound: None,
                        },
                    )
                })
                .chain([1173, 1174].into_iter().map(|id| {
                    (
                        id,
                        Definition {
                            animation: 5389,
                            animation_ticks: 4,
                            sound: Some(356),
                        },
                    )
                }))
                .collect(),
            bones: 526,
            chickens: [
                (
                    1173,
                    Chicken {
                        raw: 2138,
                        feather: 314,
                    },
                ),
                (
                    1174,
                    Chicken {
                        raw: 2138,
                        feather: 314,
                    },
                ),
            ]
            .into(),
        };

        (
            FARM.into_iter()
                .map(|(s, id)| (s.into(), id.into()))
                .collect(),
            [
                ("bones".into(), 526),
                ("raw_chicken".into(), 2138),
                ("feather".into(), 314),
            ]
            .into(),
            d,
            death,
        )
    }

    #[test]

    fn farm_chicken_catalog_adds_only_seven_and_preserves_base_kernel() {
        let (n, i, d, death) = fixture();
        let x = farm_chicken_drops(&n, &i, &sequences(), &d, &death).unwrap();
        assert_eq!(x.drops.len(), 7);
        assert_eq!(x.deaths.len(), 7);
        for v in x.deaths.values() {
            assert_eq!(
                (v.animation, v.animation_ticks, v.sound),
                (5389, 4, Some(356))
            );
        }
        for id in [1173, 1174, 3661, 3662, 9488] {
            assert!(!x.drops.contains_key(&id));
        }
        for p in x.drops.values() {
            assert_eq!((p.raw, p.feather), (2138, 314));
        }
        assert_eq!(death.chickens.len(), 2);
    }

    #[test]

    fn farm_chicken_catalog_refuses_identity_profile_lifecycle_and_material_drift() {
        for (_, id) in FARM {
            for case in 0..21 {
                let (mut n, i, mut d, mut death) = fixture();
                match case {
                    0 => {
                        n.retain(|_, v| *v != i32::from(id));
                    }
                    1 => d.npcs.get_mut(&id.into()).unwrap().name = "Quest chicken".into(),
                    2 => d.npcs.get_mut(&id.into()).unwrap().stats[3] = 10,
                    3 => d.npcs.get_mut(&id.into()).unwrap().size = 2,
                    4 => d.npcs.get_mut(&id.into()).unwrap().ops[1] = None,
                    5 => d.npcs.get_mut(&id.into()).unwrap().ops[0] = Some("Talk-to".into()),
                    6 => d.npcs.get_mut(&id.into()).unwrap().content_group = 24,
                    7 => d.npcs.get_mut(&id.into()).unwrap().timer = 1,
                    8 => d.npcs.get_mut(&id.into()).unwrap().restrict = Restrict::Blocked,
                    9 => d.npcs.get_mut(&id.into()).unwrap().default_mode = 0,
                    10 => d.npcs.get_mut(&id.into()).unwrap().wander_range = 3,
                    11 => d.npcs.get_mut(&id.into()).unwrap().multi.push(Some(1173)),
                    12 => {
                        death.definitions.remove(&id);
                    }
                    13 => death.definitions.get_mut(&id).unwrap().animation = 5389,
                    14 => death.definitions.get_mut(&id).unwrap().animation_ticks = 0,
                    15 => death.definitions.get_mut(&id).unwrap().sound = Some(356),
                    16 => {
                        death.chickens.insert(
                            id,
                            Chicken {
                                raw: 2138,
                                feather: 314,
                            },
                        );
                    }
                    17 => {
                        d.npcs.remove(&id.into());
                    }
                    18 => d.npcs.get_mut(&id.into()).unwrap().animations[0] = 65535,
                    19 => {
                        d.npcs
                            .get_mut(&id.into())
                            .unwrap()
                            .params
                            .insert(65502, gielinor_cache::definitions::Param::Integer(836));
                    }
                    _ => d.npcs.get_mut(&1174).unwrap().animations[2] = 65535,
                }

                assert!(
                    farm_chicken_drops(&n, &i, &sequences(), &d, &death).is_err(),
                    "farm{id}drift{case}"
                );
            }
        }

        for id in [526, 2138, 314] {
            for case in 0..8 {
                let (n, mut i, mut d, death) = fixture();
                match case {
                    0 => {
                        i.retain(|_, v| *v != id);
                    }
                    1 => d.items.get_mut(&(id as u32)).unwrap().members = true,
                    2 => {
                        let t = d.items.get_mut(&(id as u32)).unwrap();
                        t.stackable = !t.stackable;
                    }
                    3 => d.items.get_mut(&(id as u32)).unwrap().wear[0] = 3,
                    4 => d.items.get_mut(&(id as u32)).unwrap().dummy = 1,
                    5 => d.items.get_mut(&(id as u32)).unwrap().cert.template = 799,
                    6 => d.items.get_mut(&(id as u32)).unwrap().placeholder.template = 14401,
                    _ => {
                        d.items.remove(&(id as u32));
                    }
                }
                assert!(
                    farm_chicken_drops(&n, &i, &sequences(), &d, &death).is_err(),
                    "material{id}drift{case}"
                );
            }
        }

        for case in 0..6 {
            let (n, i, mut d, mut death) = fixture();
            let mut seq = sequences();
            match case {
                0 => {
                    seq.remove("lore_chicken_death");
                }
                1 => death.definitions.get_mut(&1173).unwrap().animation_ticks = 2,
                2 => death.definitions.get_mut(&1174).unwrap().sound = None,
                3 => {
                    d.npcs.get_mut(&1174).unwrap().params.remove(&65502);
                }
                4 => {
                    d.npcs.remove(&1173);
                }
                _ => d.npcs.get_mut(&1173).unwrap().name = "Other".into(),
            }
            assert!(farm_chicken_drops(&n, &i, &seq, &d, &death).is_err());
        }
        for case in 0..3 {
            let (n, i, d, mut death) = fixture();
            match case {
                0 => death.bones = 527,
                1 => {
                    death.chickens.remove(&1173);
                }
                _ => death.chickens.get_mut(&1174).unwrap().raw = 2140,
            }
            assert!(farm_chicken_drops(&n, &i, &sequences(), &d, &death).is_err());
        }
    }
}
