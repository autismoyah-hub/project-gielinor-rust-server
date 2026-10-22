//! No unconditional remains/main loot for the attested small-spider family.
//! Conditional clues/bags and special-area/quest behavior are separate.
use super::{Definitions, Symbols};
use gielinor_world::{
    npc::{BlockWalk, Restrict},
    npc_death::{Config, Definition, DropPolicy},
    npc_loot::Profile,
};
use std::collections::BTreeMap;
const TYPES: [(&str, u16, [i32; 6]); 3] = [
    ("spider", 3019, [1, 0, 0, 2, 0, 0]),
    ("sos_pest_spider", 2478, [21, 21, 21, 22, 1, 1]),
    ("upass_little_spider", 4561, [1, 0, 0, 2, 0, 0]),
];
pub struct SpiderDrops {
    pub profiles: BTreeMap<u16, Profile>,
    pub deaths: BTreeMap<u16, Definition>,
}
pub fn spider_drops(
    npcs: &Symbols,
    sequences: &Symbols,
    d: &Definitions,
    death: &Config,
) -> Result<SpiderDrops, String> {
    for (symbol, id) in [
        ("small_spider_update_ready", 6247),
        ("small_spider_update_walk", 6248),
        ("small_spider_update_death", 6251),
    ] {
        if sequences.get(symbol) != Some(&id) {
            return Err("Unattested small-spider family sequence".into());
        }
    }
    let baseline = *death
        .definitions
        .get(&3019)
        .ok_or("Missing ordinary spider lifecycle")?;
    let base = d
        .npcs
        .get(&3019)
        .ok_or("Missing ordinary spider baseline")?;
    if (baseline.animation, baseline.animation_ticks, baseline.sound) != (6251, 3, Some(3608))
        || base.params.get(&65502) != Some(&gielinor_cache::definitions::Param::Integer(6251))
        || base.params.get(&65506) != Some(&gielinor_cache::definitions::Param::Integer(3608))
    {
        return Err("Unattested ordinary small-spider baseline lifecycle".into());
    }
    let mut profiles = BTreeMap::new();
    let mut deaths = BTreeMap::new();
    for &(symbol, id, stats) in &TYPES {
        let t = d
            .npcs
            .get(&u32::from(id))
            .ok_or("Missing source spider profile")?;
        let observed = death
            .definitions
            .get(&id)
            .ok_or("Missing spider lifecycle")?;
        if npcs.get(symbol) != Some(&i32::from(id))
            || t.name != "Spider"
            || t.size != 1
            || t.stats != stats
            || t.ops != [None, Some("Attack".into()), None, None, None]
            || t.default_mode != 1
            || t.timer != -1
            || t.wander_range != 5
            || t.content_group != -1
            || t.restrict != Restrict::Normal
            || !matches!(t.block_walk, BlockWalk::Npc)
            || !t.multi.is_empty()
            || t.multi_varp.is_some()
            || t.multi_varbit.is_some()
            || t.multi_default.is_some()
            || t.animations != [6247, 65535, 6248, 6248, 65535, 65535, 65535]
            || death.chickens.contains_key(&id)
        {
            return Err("Unattested registered small-spider profile".into());
        }
        let supported = match t.params.get(&65502) {
            None => {
                (observed.animation, observed.animation_ticks, observed.sound) == (836, 2, None)
            }
            Some(gielinor_cache::definitions::Param::Integer(6251)) => {
                (observed.animation, observed.animation_ticks, observed.sound)
                    == (baseline.animation, baseline.animation_ticks, baseline.sound)
            }
            _ => false,
        };
        if !supported {
            return Err("Conflicting small-spider lifecycle".into());
        }
        // Preserve the explicit ordinary baseline. The two same-rig variants omit
        // death_anim and inherit human836; copy only this attested family baseline.
        deaths.insert(id, baseline);
        profiles.insert(
            id,
            Profile::new(DropPolicy::Spider, vec![], None)
                .map_err(|e| format!("Invalid empty spider loot profile: {e:?}"))?,
        );
    }
    Ok(SpiderDrops { profiles, deaths })
}
#[cfg(test)]
mod tests {
    use super::*;
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
        let mut n = Symbols::new();
        let mut defs = BTreeMap::new();
        for &(symbol, id, stats) in &TYPES {
            let mut t = gielinor_cache::npcs::NpcType::decode(&[0]).unwrap();
            t.name = "Spider".into();
            t.size = 1;
            t.stats = stats;
            t.ops[1] = Some("Attack".into());
            t.wander_range = 5;
            t.animations = [6247, 65535, 6248, 6248, 65535, 65535, 65535];
            let end = if id == 3019 {
                t.params
                    .insert(65502, gielinor_cache::definitions::Param::Integer(6251));
                t.params
                    .insert(65506, gielinor_cache::definitions::Param::Integer(3608));
                Definition {
                    animation: 6251,
                    animation_ticks: 3,
                    sound: Some(3608),
                }
            } else {
                Definition {
                    animation: 836,
                    animation_ticks: 2,
                    sound: None,
                }
            };
            d.npcs.insert(u32::from(id), t);
            n.insert(symbol.into(), i32::from(id));
            defs.insert(id, end);
        }
        (
            n,
            [
                ("small_spider_update_ready".into(), 6247),
                ("small_spider_update_walk".into(), 6248),
                ("small_spider_update_death".into(), 6251),
            ]
            .into(),
            d,
            Config {
                definitions: defs,
                bones: 526,
                chickens: Default::default(),
            },
        )
    }
    #[test]
    fn three_source_types_have_no_normal_loot_and_only_matching_family_death() {
        let (n, s, d, c) = fixture();
        let x = spider_drops(&n, &s, &d, &c).unwrap();
        assert_eq!(x.profiles.len(), 3);
        assert_eq!(x.deaths.len(), 3);
        for id in [2478, 3019, 4561] {
            let p = &x.profiles[&id];
            assert_eq!(p.policy(), DropPolicy::Spider);
            assert_eq!(p.bound(), None);
            assert_eq!(p.products(None), Some(vec![]));
            assert_eq!(p.products(Some(0)), None);
            assert_eq!(
                (
                    x.deaths[&id].animation,
                    x.deaths[&id].animation_ticks,
                    x.deaths[&id].sound
                ),
                (6251, 3, Some(3608))
            );
        }
        assert_eq!(
            (
                c.definitions[&3019].animation,
                c.definitions[&3019].animation_ticks,
                c.definitions[&3019].sound
            ),
            (6251, 3, Some(3608))
        );
    }
    #[test]
    fn source_identity_rig_lifecycle_and_baseline_drift_fail_closed() {
        for raw in [2478, 3019, 4561] {
            for case in 0..8 {
                let (mut n, s, mut d, mut c) = fixture();
                let t = d.npcs.get_mut(&(raw as u32)).unwrap();
                match case {
                    0 => {
                        n.retain(|_, id| *id != raw);
                    }
                    1 => t.stats[3] += 1,
                    2 => t.size = 2,
                    3 => t.ops[2] = Some("Talk-to".into()),
                    4 => t.animations[0] = 5326,
                    5 => t.multi.push(Some(3019)),
                    6 => {
                        t.params
                            .insert(65502, gielinor_cache::definitions::Param::Integer(5329));
                    }
                    _ => {
                        c.chickens.insert(
                            raw as u16,
                            gielinor_world::npc_death::Chicken {
                                raw: 2138,
                                feather: 314,
                            },
                        );
                    }
                }
                assert!(spider_drops(&n, &s, &d, &c).is_err(), "raw{raw}case{case}");
            }
        }
        for case in 0..6 {
            let (n, mut s, mut d, mut c) = fixture();
            match case {
                0 => {
                    s.insert("small_spider_update_death".into(), 5329);
                }
                1 => {
                    d.npcs.get_mut(&3019).unwrap().params.remove(&65502);
                }
                2 => {
                    d.npcs.get_mut(&3019).unwrap().params.remove(&65506);
                }
                3 => c.definitions.get_mut(&3019).unwrap().animation_ticks = 2,
                4 => c.definitions.get_mut(&2478).unwrap().sound = Some(1),
                _ => {
                    c.definitions.remove(&4561);
                }
            }
            assert!(spider_drops(&n, &s, &d, &c).is_err(), "baseline{case}");
        }
    }
}
