//! Ordinary adult bears: explicit guarantees and target bear-family lifecycle.
//! The named sequence adaptation needs native presentation verification.
use super::{Definitions, Symbols};
use gielinor_cache::sequences::SequenceType;
use gielinor_world::{
    npc::{BlockWalk, Restrict},
    npc_death::{Config, Definition, DropPolicy},
    npc_loot::{Product, Profile},
};
use std::collections::BTreeMap;
const ADULTS: [(&str, u16, &str, [i32; 6]); 2] = [
    ("brownbear", 2838, "Grizzly bear", [17, 15, 18, 27, 1, 1]),
    ("darkbear", 2839, "Black bear", [15, 13, 16, 25, 1, 1]),
];
const PRODUCTS: [(&str, u16, &str); 3] = [
    ("bones", 526, "Bones"),
    ("fur", 948, "Bear fur"),
    ("raw_bear_meat", 2136, "Raw bear meat"),
];
const DEATH_DELAYS: [i16; 17] = [5, 6, 7, 8, 9, 8, 7, 4, 4, 4, 4, 6, 4, 4, 4, 4, 100];
pub struct BearDrops {
    pub profiles: BTreeMap<u16, Profile>,
    pub deaths: BTreeMap<u16, Definition>,
}
pub fn bear_drops(
    npcs: &Symbols,
    items: &Symbols,
    sequences: &Symbols,
    d: &Definitions,
    death: &Config,
    sequence_types: &BTreeMap<u32, SequenceType>,
) -> Result<BearDrops, String> {
    if death.bones != 526 {
        return Err("Unattested ordinary bone identity".into());
    }
    for &(symbol, id, name) in &PRODUCTS {
        let t = d.items.get(&u32::from(id)).ok_or("Missing bear product")?;
        if items.get(symbol) != Some(&i32::from(id))
            || t.name != name
            || t.members
            || t.stackable
            || t.wear != [-1, -1, -1]
            || t.dummy != -1
            || t.cert.template != 0
            || t.placeholder.template != 0
            || t.transform.template != 0
            || !t.objvars.is_empty()
        {
            return Err("Unattested ordinary bear product".into());
        }
    }
    for (symbol, id) in [
        ("bear_rework_ready", 4919),
        ("bear_rework_walk", 4923),
        ("bear_rework_death", 4929),
    ] {
        if sequences.get(symbol) != Some(&id) {
            return Err("Unattested adult bear sequence".into());
        }
    }
    let seq = sequence_types
        .get(&4929)
        .ok_or("Missing adult bear death sequence")?;
    if seq.delays != DEATH_DELAYS || seq.tick_duration() != 4 {
        return Err("Unattested adult bear death cadence".into());
    }
    let mut profiles = BTreeMap::new();
    let mut deaths = BTreeMap::new();
    for &(symbol, id, name, stats) in &ADULTS {
        let t = d
            .npcs
            .get(&u32::from(id))
            .ok_or("Missing ordinary adult bear")?;
        let observed = death.definitions.get(&id).ok_or("Missing bear lifecycle")?;
        if npcs.get(symbol) != Some(&i32::from(id))
            || t.name != name
            || t.size != 2
            || t.stats != stats
            || t.ops != [None, Some("Attack".into()), None, None, None]
            || t.content_group != -1
            || t.default_mode != 1
            || t.timer != -1
            || t.wander_range != 5
            || t.restrict != Restrict::Normal
            || !matches!(t.block_walk, BlockWalk::Npc)
            || !t.multi.is_empty()
            || t.multi_varp.is_some()
            || t.multi_varbit.is_some()
            || t.multi_default.is_some()
            || t.animations != [4919, 65535, 4923, 4923, 65535, 65535, 65535]
            || death.chickens.contains_key(&id)
        {
            return Err("Unattested ordinary adult bear profile".into());
        }
        // The target currently omits death_anim and resolves to generic human836.
        // Select the named adult rework sequence only for this exact ready/walk rig.
        // No sound is inferred, and cub/old-bear rigs are deliberately excluded.
        let supported = match t.params.get(&65502) {
            None => {
                (observed.animation, observed.animation_ticks, observed.sound) == (836, 2, None)
            }
            Some(gielinor_cache::definitions::Param::Integer(4929)) => {
                (observed.animation, observed.animation_ticks, observed.sound) == (4929, 4, None)
            }
            _ => false,
        };
        if !supported {
            return Err("Conflicting adult bear lifecycle".into());
        }
        deaths.insert(
            id,
            Definition {
                animation: 4929,
                animation_ticks: 4,
                sound: None,
            },
        );
        profiles.insert(
            id,
            Profile::new(
                DropPolicy::Bear,
                PRODUCTS
                    .iter()
                    .map(|&(_, item, _)| Product { item, count: 1 })
                    .collect(),
                None,
            )
            .map_err(|e| format!("Invalid bear guarantees: {e:?}"))?,
        );
    }
    Ok(BearDrops { profiles, deaths })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (
        Symbols,
        Symbols,
        Symbols,
        Definitions,
        Config,
        BTreeMap<u32, SequenceType>,
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
        let mut i = Symbols::new();
        let mut deaths = BTreeMap::new();
        for &(symbol, id, name, stats) in &ADULTS {
            let mut t = gielinor_cache::npcs::NpcType::decode(&[0]).unwrap();
            t.name = name.into();
            t.size = 2;
            t.stats = stats;
            t.ops[1] = Some("Attack".into());
            t.wander_range = 5;
            t.animations = [4919, 65535, 4923, 4923, 65535, 65535, 65535];
            d.npcs.insert(u32::from(id), t);
            n.insert(symbol.into(), i32::from(id));
            deaths.insert(
                id,
                Definition {
                    animation: 836,
                    animation_ticks: 2,
                    sound: None,
                },
            );
        }
        for &(symbol, id, name) in &PRODUCTS {
            let mut t = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            t.name = name.into();
            d.items.insert(u32::from(id), t);
            i.insert(symbol.into(), i32::from(id));
        }
        let mut seq = SequenceType::decode(&[0]).unwrap();
        seq.delays = DEATH_DELAYS.to_vec();
        (
            n,
            i,
            [
                ("bear_rework_ready".into(), 4919),
                ("bear_rework_walk".into(), 4923),
                ("bear_rework_death".into(), 4929),
            ]
            .into(),
            d,
            Config {
                definitions: deaths,
                bones: 526,
                chickens: Default::default(),
            },
            [(4929, seq)].into(),
        )
    }
    #[test]
    fn ordinary_adults_use_explicit_no_roll_guarantees_and_matching_rig_lifecycle() {
        let (n, i, s, d, c, t) = fixture();
        let b = bear_drops(&n, &i, &s, &d, &c, &t).unwrap();
        assert_eq!(b.profiles.keys().copied().collect::<Vec<_>>(), [2838, 2839]);
        for id in [2838, 2839] {
            let p = &b.profiles[&id];
            assert_eq!(p.bound(), None);
            assert_eq!(p.policy(), DropPolicy::Bear);
            assert_eq!(p.products(None), Some(vec![(526, 1), (948, 1), (2136, 1)]));
            assert_eq!(p.products(Some(0)), None);
            assert_eq!(
                (
                    b.deaths[&id].animation,
                    b.deaths[&id].animation_ticks,
                    b.deaths[&id].sound
                ),
                (4929, 4, None)
            );
        }
        for id in [3423, 3908, 3909] {
            assert!(!b.profiles.contains_key(&id));
        }
    }
    #[test]
    fn bear_material_rig_source_lifecycle_and_sequence_drift_fail_closed() {
        for id in [526, 948, 2136] {
            for case in 0..9 {
                let (n, mut i, s, mut d, c, t) = fixture();
                let v = d.items.get_mut(&id).unwrap();
                match case {
                    0 => v.name = "Unknown".into(),
                    1 => v.members = true,
                    2 => v.stackable = true,
                    3 => v.wear[0] = 3,
                    4 => v.dummy = 1,
                    5 => v.cert.template = 799,
                    6 => v.placeholder.template = 14401,
                    7 => v.transform.template = 1,
                    _ => {
                        i.retain(|_, v| *v != id as i32);
                    }
                }
                assert!(
                    bear_drops(&n, &i, &s, &d, &c, &t).is_err(),
                    "item{id}case{case}"
                );
            }
        }
        for case in 0..12 {
            let (mut n, i, mut s, mut d, mut c, mut t) = fixture();
            match case {
                0 => {
                    n.insert("brownbear".into(), 3423);
                }
                1 => d.npcs.get_mut(&2838).unwrap().stats[3] = 35,
                2 => d.npcs.get_mut(&2839).unwrap().size = 1,
                3 => d.npcs.get_mut(&2838).unwrap().ops[2] = Some("Talk-to".into()),
                4 => d.npcs.get_mut(&2839).unwrap().animations[0] = 4920,
                5 => {
                    d.npcs
                        .get_mut(&2838)
                        .unwrap()
                        .params
                        .insert(65502, gielinor_cache::definitions::Param::Integer(44));
                }
                6 => c.definitions.get_mut(&2839).unwrap().sound = Some(1),
                7 => {
                    s.insert("bear_rework_death".into(), 44);
                }
                8 => t.get_mut(&4929).unwrap().delays = vec![],
                9 => {
                    c.chickens.insert(
                        2838,
                        gielinor_world::npc_death::Chicken {
                            raw: 2138,
                            feather: 314,
                        },
                    );
                }
                10 => d.npcs.get_mut(&2838).unwrap().multi.push(Some(3908)),
                _ => c.bones = 532,
            }
            assert!(bear_drops(&n, &i, &s, &d, &c, &t).is_err(), "drift{case}");
        }
    }
}
