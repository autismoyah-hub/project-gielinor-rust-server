//! Clay rocks and infinite rune essence rocks: OpenRune or-cache `Mining.kt` rows
//! `mining_clayrock` and `mining_essence` (deplete mechanic 3, `isInfinite`), with Wiki
//! Clay rocks (rev 15372445) and Rune Essence (rock) (rev 15364416).
//!
//! - Clay: level 1, 5 XP, rates 128/400, depletes to rocks2 and respawns after 2 ticks.
//! - Essence: level 1, 5 XP, rates 256/256 (every attempt succeeds), never depletes.
//! - Intentional adaptation: members with Mining 30+ mine pure essence (Wiki Pure essence
//!   rev 15325415, Rune essence rev 15325416). OpenRune always gives rune essence.
//!
//! Clay draws the gem pre-roll (OpenRune rollGemPreTable eligibility; the Wiki Clay rocks
//! page lists the pre-roll); the infinite essence rock does not. The essence mine teleport,
//! the bracelet of clay and soft clay rocks are not part of this slice.
use super::{Definitions, Symbols};
use gielinor_world::mining::{Essence, Rock};
use std::collections::BTreeMap;

pub const CLAY_ROCKS: [(&str, u32); 2] = [("clayrock1", 11362), ("clayrock2", 11363)];
pub const ESSENCE_ROCK: (&str, u32) = ("blankrunestone", 34773);
pub const DEPLETED: (&str, u32) = ("rocks2", 11391);
/// (symbol, id, cache name).
pub const CLAY: (&str, i32, &str) = ("clay", 434, "Clay");
pub const RUNE_ESSENCE: (&str, i32, &str) = ("blankrune", 1436, "Rune essence");
pub const PURE_ESSENCE: (&str, i32, &str) = ("blankrune_high", 7936, "Pure essence");
pub const PURE_LEVEL: i32 = 30;

fn product(objects: &Symbols, d: &Definitions, (symbol, id, name): (&str, i32, &str)) -> Result<String, String> {
    let item = d.items.get(&(id as u32));
    if objects.get(symbol) != Some(&id)
        || item.is_none_or(|t| {
            t.name != name
                || t.members
                || t.stackable
                || t.dummy == 1
                || t.cert.template != 0
                || t.placeholder.template != 0
                || t.transform.template != 0
        })
    {
        return Err(format!("clay/essence mining: unattested obj {symbol} ({id})"));
    }
    Ok(name.into())
}

fn rock(locations: &Symbols, d: &Definitions, (symbol, id): (&str, u32), name: &str) -> Result<(), String> {
    let loc = d.locations.get(&id);
    if locations.get(symbol) != Some(&(id as i32))
        || loc.is_none_or(|l| {
            l.name != name
                || l.ops[0].as_deref() != Some("Mine")
                || !l.multi.is_empty()
                || l.multi_varbit != -1
                || l.multi_varp != -1
        })
    {
        return Err(format!("clay/essence mining: unattested loc {symbol} ({id})"));
    }
    Ok(())
}

/// Gem pre-roll rocks among `rocks`: OpenRune rollGemPreTable skips gem rocks, rocks
/// without an ore, infinite rocks and rows without a clue chance. Both rows here have an
/// ore and a clue chance, so only the essence rock (infinite) is excluded.
pub fn pre_roll_rocks(rocks: &BTreeMap<u32, Rock>) -> std::collections::BTreeSet<u32> {
    rocks.iter().filter(|(_, r)| !r.infinite()).map(|(&id, _)| id).collect()
}

/// Clay and essence rock profiles, plus the pure essence rule for the essence rock.
pub fn clay_essence_mining(
    objects: &Symbols,
    locations: &Symbols,
    d: &Definitions,
) -> Result<(BTreeMap<u32, Rock>, Essence), String> {
    if locations.get(DEPLETED.0) != Some(&(DEPLETED.1 as i32)) || !d.locations.contains_key(&DEPLETED.1) {
        return Err("clay/essence mining: missing depleted rock".into());
    }
    let mut rocks = BTreeMap::new();
    let clay = product(objects, d, CLAY)?;
    for &target in &CLAY_ROCKS {
        rock(locations, d, target, "Clay rocks")?;
        rocks.insert(
            target.1,
            Rock {
                members: false,
                restore: None,
                level: 1,
                product: CLAY.1,
                product_name: clay.clone(),
                xp: 5.0,
                depleted: DEPLETED.1,
                respawn: 2,
                rates: (128, 400),
            },
        );
    }
    rock(locations, d, ESSENCE_ROCK, "Rune Essence")?;
    rocks.insert(
        ESSENCE_ROCK.1,
        Rock {
            members: false,
            restore: None,
            level: 1,
            product: RUNE_ESSENCE.1,
            product_name: product(objects, d, RUNE_ESSENCE)?,
            xp: 5.0,
            depleted: ESSENCE_ROCK.1,
            respawn: 0,
            rates: (256, 256),
        },
    );
    let essence = Essence {
        rocks: [ESSENCE_ROCK.1].into(),
        pure: PURE_ESSENCE.1,
        pure_name: product(objects, d, PURE_ESSENCE)?,
        pure_level: PURE_LEVEL,
    };
    essence.validate(&rocks).map_err(String::from)?;
    Ok((rocks, essence))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Symbols, Symbols, Definitions) {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut o = Symbols::new();
        for (symbol, id, name) in [CLAY, RUNE_ESSENCE, PURE_ESSENCE] {
            let mut item = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            item.name = name.into();
            d.items.insert(id as u32, item);
            o.insert(symbol.into(), id);
        }
        let mut l = Symbols::new();
        for ((symbol, id), name) in CLAY_ROCKS
            .iter()
            .map(|&r| (r, "Clay rocks"))
            .chain([(ESSENCE_ROCK, "Rune Essence"), (DEPLETED, "Rocks")])
        {
            let mut b = vec![2];
            b.extend_from_slice(name.as_bytes());
            b.extend_from_slice(b"\0\x1eMine\0\0");
            d.locations.insert(id, gielinor_cache::locations::LocationType::decode(&b).unwrap());
            l.insert(symbol.into(), id as i32);
        }
        (o, l, d)
    }
    #[test]
    fn clay_depletes_and_essence_is_infinite_with_wiki_values() {
        let (o, l, d) = fixture();
        let (rocks, essence) = clay_essence_mining(&o, &l, &d).unwrap();
        assert_eq!(rocks.keys().copied().collect::<Vec<_>>(), [11362, 11363, 34773]);
        for id in [11362, 11363] {
            let r = &rocks[&id];
            assert_eq!((r.level, r.xp, r.product, r.depleted, r.respawn, r.rates), (1, 5.0, 434, 11391, 2, (128, 400)));
            assert!(!r.infinite());
        }
        let e = &rocks[&34773];
        assert_eq!((e.level, e.xp, e.product, e.depleted, e.respawn, e.rates), (1, 5.0, 1436, 34773, 0, (256, 256)));
        assert!(e.infinite());
        assert_eq!(pre_roll_rocks(&rocks), [11362, 11363].into(), "clay draws the gem pre-roll, essence does not");
        // Wiki rule: members with Mining 30+ get pure essence; everyone else rune essence.
        for (member, level, item) in [(false, 29, 1436), (false, 30, 1436), (true, 29, 1436), (true, 30, 7936), (true, 99, 7936)] {
            assert_eq!(essence.product(e, member, level).0, item, "member={member} level={level}");
        }
    }
    #[test]
    fn identity_drift_fails_closed() {
        for case in 0..7 {
            let (mut o, mut l, mut d) = fixture();
            match case {
                0 => {
                    o.insert("blankrune".into(), 7936);
                }
                1 => d.items.get_mut(&7936).unwrap().name = "Rune essence".into(),
                2 => d.items.get_mut(&434).unwrap().stackable = true,
                3 => {
                    l.insert("clayrock2".into(), 11364);
                }
                4 => d.locations.get_mut(&34773).unwrap().ops[0] = Some("Prospect".into()),
                5 => {
                    l.remove("rocks2");
                }
                _ => d.items.get_mut(&1436).unwrap().cert.template = 799,
            }
            assert!(clay_essence_mining(&o, &l, &d).is_err(), "drift{case}");
        }
    }
}
